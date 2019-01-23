//! Implementation of `DoHResolver` for the DoH JSON Protocol.

use doh_json::{response::*, provider::DoHJsonProvider};
use core::{provider::*, resolver::*, response::*};
use dns::protocol::{DnsMessage, DnsMessageType};
use threadpool::{ThreadPool, Builder as ThreadPoolBuilder};
use num_cpus;
use curl::easy::{Easy as CurlEasy, HttpVersion as CurlHttpVersion, List as CurlList};
use http::{Version as HttpVersion, Request as HttpRequest, HeaderMap as HttpHeaderMap};
use std::{sync::mpsc::channel, time::Duration};

type Result<T> = std::result::Result<T, DoHResolutionError>;

/// DNS-over-HTTPS resolver that implements the DoH JSON protocol
///
/// This is an "Authoritative" resolver: responses are always coming from the given provider,
/// not the cache. As such, responses will always have the "authoritative" bit on.
struct DoHJsonResolver<'a> {
  provider: &'a DoHJsonProvider,
  pool: ThreadPool
}

impl <'a> DoHJsonResolver<'a> {

  fn new(provider: &DoHJsonProvider) -> DoHJsonResolver {
    let pool = ThreadPoolBuilder::new()
      .num_threads(num_cpus::get())
      .thread_name("DoHJsonResolver Thread Pool".into())
      .build();

    DoHJsonResolver {
      provider,
      pool,
    }
  }

}

impl <'a> DoHResolver for DoHJsonResolver<'a> {

  fn resolve_message_query(&self, req_dns_msg: &DnsMessage) -> Result<DnsMessage> {
    // Begin preparing response DNS Message
    let mut res_dns_msg = DnsMessage::new();
    res_dns_msg.set_id(req_dns_msg.id());
    res_dns_msg.set_op_code(req_dns_msg.op_code());
    res_dns_msg.set_message_type(DnsMessageType::Response);
    res_dns_msg.set_authoritative(true);

    // Execute all queries in parallel
    let queries_count = req_dns_msg.queries().len();
    let (tx, rx) = channel();
    for query in req_dns_msg.queries() {
      let tx = tx.clone();
      let provider = self.provider.clone();
      let query = query.clone();

      self.pool.execute(move || {
        let req_http = provider.build_http_request(&query).unwrap();
        let res_doh_result = execute_http_request(req_http);

        tx.send(res_doh_result).expect("Couldn't deliver HTTP request `Result<DoHJsonResponse>`: this should never happen!");
      });
    }

    // Wait for all the parallel requests to return a `Result`,
    // then apply to the `DnsMessage` response the ones that have succeeded
    rx.iter()
      .take(queries_count)
      .for_each(|res_doh_result| {
        match res_doh_result {
          Ok(res_doh) => {
            // TODO Provide the correct edns_client_subnet_prefix if present
            res_doh.apply(0, &mut res_dns_msg);
          },
          Err(err) => error!("A DoH JSON HTTP Request failed: {}", err),
        };
      });

    Ok(res_dns_msg)
  }

}

/// Converts an `http::Version` to the corresponding value in `curl::HttpVersion`
///
/// # Parameters
///
/// * `version`: The HTTP Version to convert
fn http_version_to_curl(version: HttpVersion) -> CurlHttpVersion {
  match version {
    HttpVersion::HTTP_2 => CurlHttpVersion::V2,
    HttpVersion::HTTP_10 => CurlHttpVersion::V10,
    HttpVersion::HTTP_11 | _ => CurlHttpVersion::V11,
  }
}

/// Converts an `http:HeaderMap` to a `curl::List` (used to carry headers)
///
/// # Parameters
///
/// * `header_map`: A map of headers
fn http_headers_to_curl(header_map: &HttpHeaderMap) -> CurlList {
  let mut curl_headers = CurlList::new();

  for (name, value) in header_map.iter() {
    curl_headers.append(format!("{}: {}", name, value.to_str().unwrap()).as_ref()).unwrap();
  }

  curl_headers
}

/// Executes a (synchronous) HTTP Request and return a `DoHJsonResponse`
///
/// # Parameters
///
/// * `req_http`: An `http::Request`, created by a `DoHJsonProvider`, that will return a parse-able `DoHJsonResponse`
fn execute_http_request(req_http: HttpRequest<()>) -> Result<DoHJsonResponse> {
  // Init a cURL request and a buffer to store the response
  let mut req_curl = CurlEasy::new();
  let mut res_curl_buf: Vec<u8> = Vec::new();

  // Setup the cURL Request by adapting the given HTTP Request
  req_curl.timeout(Duration::from_secs(60))?;
  req_curl.http_version(http_version_to_curl(req_http.version()))?;
  req_curl.url(format!("{}", req_http.uri()).as_ref())?;
  req_curl.http_headers(http_headers_to_curl(req_http.headers()))?;

  // Execute the request and wait for data to be written in the response buffer
  {
    let mut req_curl_transfer = req_curl.transfer();
    req_curl_transfer.write_function(|data| {
      res_curl_buf.extend_from_slice(data);
      Ok(data.len())
    })?;
    req_curl_transfer.perform()?;
  }

  // Parse the response buffer into a DoHJsonResponse
  DoHJsonResponse::from_slice(&res_curl_buf)
    .map_err(|serde_error| DoHResolutionError::from(serde_error))
}

#[cfg(test)]
mod test {
  use super::*;
  use doh_json::provider::{self, DoHJsonProvider};
  use std::{io::Read, fs::File, path::Path};

  fn read_file_to_vec<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    buf
  }

  #[test]
  fn should_resolve_example_dot_com() {
    let providers = DoHJsonProvider::defaults();
    let provider = providers.get(provider::PROVIDER_NAME_GOOGLE).unwrap();

    let resolver = DoHJsonResolver::new(provider);

    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_A-example.com-packet.bin");
    let dns_msg_req = DnsMessage::from_vec(&buf).unwrap();

    let dns_msg_res_result = resolver.resolve_message_query(&dns_msg_req);
    println!("{:#?}", dns_msg_res_result);
  }
}