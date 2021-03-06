//! Implementation of `DoHResolver` for the DoH JSON Protocol.

use super::{response::*, provider::DoHJsonProvider};
use crate::core::{provider::*, resolver::*, response::*};
use crate::dns::protocol::{DnsMessage, DnsMessageType};

use log::*;
use threadpool::{ThreadPool, Builder as ThreadPoolBuilder};
use num_cpus;
use curl::easy::{Easy as CurlEasy, HttpVersion as CurlHttpVersion, List as CurlList};
use http::{Version as HttpVersion, Request as HttpRequest, HeaderMap as HttpHeaderMap};
use crossbeam_channel::bounded;

use std::time::Duration;

const DOH_JSON_RESOLVER_THREAD_NAME: &'static str = "doh_json_resolver_thread";

type Result<T> = std::result::Result<T, DoHResolutionError>;

/// DNS-over-HTTPS resolver that implements the DoH JSON protocol
///
/// This is an "Authoritative" resolver: responses are always coming from the given provider,
/// not the cache. As such, responses will always have the "authoritative" bit on.
#[derive(Debug, Clone)]
pub struct DoHJsonResolver {
  provider: DoHJsonProvider,
  pool: ThreadPool
}

impl DoHJsonResolver {

  pub fn new(provider: DoHJsonProvider) -> DoHJsonResolver {
    let pool = ThreadPoolBuilder::new()
      .num_threads(num_cpus::get())
      .thread_name(DOH_JSON_RESOLVER_THREAD_NAME.into())
      .build();

    DoHJsonResolver {
      provider,
      pool,
    }
  }

}

impl DoHResolver for DoHJsonResolver {

  fn resolve_query(&self, req_dns_msg: &DnsMessage) -> Result<DnsMessage> {
    // Begin preparing response DNS Message
    let mut res_dns_msg = DnsMessage::new();
    res_dns_msg.set_id(req_dns_msg.id());
    res_dns_msg.set_op_code(req_dns_msg.op_code());
    res_dns_msg.set_message_type(DnsMessageType::Response);
    res_dns_msg.set_authoritative(true);

    // Execute all queries in parallel
    let queries_count = req_dns_msg.queries().len();
    let (tx, rx) = bounded(queries_count);
    for query in req_dns_msg.queries() {
      let tx = tx.clone();
      let provider = self.provider.clone();
      let query = query.clone();

      self.pool.execute(move || {
        let req_http = provider.build_http_request(&query).unwrap();
        let res_doh = execute_http_request(req_http);

        trace!("DoH response: {:?}", res_doh);

        tx.send(res_doh)
          .expect("Couldn't deliver HTTP request `Result<DoHJsonResponse>`: this should never happen!");
      });
    }

    // Wait for all the parallel requests to return a `Result`,
    // then apply to the `DnsMessage` response the ones that have succeeded
    rx.iter()
      .take(queries_count)
      .for_each(|res_doh_result| {
        match res_doh_result {
          Ok(res_doh) => {
            // TODO Provide the correct edns_client_subnet_prefix if present in `req_dns_msg`
            res_doh.apply(0, &mut res_dns_msg);
          },
          Err(err) => error!("A DoH JSON HTTP Request failed: {}", err),
        };
      });

    Ok(res_dns_msg)
  }

  fn box_clone(&self) -> Box<DoHResolver + Send> {
    Box::new((*self).clone())
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

  trace!("Raw DoH response: {:?}", std::str::from_utf8(&res_curl_buf).unwrap());

  // Parse the response buffer into a DoHJsonResponse
  DoHJsonResponse::from_slice(&res_curl_buf)
    .map_err(|serde_error| DoHResolutionError::from(serde_error))
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::doh_json::provider::{self, DoHJsonProvider};
  use crate::dns::protocol::{DnsMessage, DnsRecordType, DnsClass, DnsRData, dns_message_to_bytes, dns_message_from_bytes};
  use std::{io::Read, fs::File, path::Path};

  fn read_file_to_vec<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    buf
  }

  // TODO I hate every character of this function: this is only necessary because trust-dns
  //  designed DnsMessage to be truly finalized (i.e. all headers updated) at the serialization time.
  //  I'll fix this once I take out the part of trust-dns I need for this project.
  fn force_msg_finalization(dns_msg: DnsMessage) -> DnsMessage {
    let bytes = dns_message_to_bytes(&dns_msg).unwrap();
    dns_message_from_bytes(&bytes).unwrap()
  }

  #[test]
  fn should_resolve_udp_query_example_com() {
    let provider = DoHJsonProvider::available().remove(provider::PROVIDER_NAME_GOOGLE).unwrap();

    let resolver = DoHJsonResolver::new(provider);

    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_A-example.com-packet.bin");
    let dns_req = DnsMessage::from_vec(&buf).unwrap();

    let dns_res_result = resolver.resolve_query(&dns_req);
    assert!(dns_res_result.is_ok());
    let dns_res = force_msg_finalization(dns_res_result.unwrap());

    assert_eq!(dns_res.message_type(), DnsMessageType::Response);
    assert!(dns_res.id() > 0);
    assert_eq!(dns_res.query_count(), 1);
    assert_eq!(dns_res.answer_count(), 1);
    assert_eq!(dns_res.additional_count(), 0);
    assert_eq!(dns_res.name_server_count(), 0);

    assert_eq!(dns_res.queries().len(), 1);
    let dns_query = &(dns_res.queries())[0];
    assert_eq!(dns_query.query_type(), DnsRecordType::A);
    assert_eq!(dns_query.query_class(), DnsClass::IN);
    assert_eq!(dns_query.name().to_utf8(), "example.com.");

    assert_eq!(dns_res.answers().len(), 1);
    let dns_answer = &(dns_res.answers())[0];
    assert_eq!(dns_answer.record_type(), DnsRecordType::A);
    assert_eq!(dns_answer.dns_class(), DnsClass::IN);
    assert_eq!(dns_answer.name().to_utf8(), "example.com.");
    assert!(dns_answer.ttl() > 100);
    assert_eq!(dns_answer.rdata().to_record_type(), DnsRecordType::A);

    assert!(dns_res.edns().is_none());
  }

  #[test]
  fn should_resolve_udp_query_noedns_example_com() {
    let provider = DoHJsonProvider::available().remove(provider::PROVIDER_NAME_QUAD9).unwrap();

    let resolver = DoHJsonResolver::new(provider);

    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_noedns_A-example.com-packet.bin");
    let dns_req = DnsMessage::from_vec(&buf).unwrap();

    let dns_res_result = resolver.resolve_query(&dns_req);
    assert!(dns_res_result.is_ok());
    let dns_res = force_msg_finalization(dns_res_result.unwrap());

    assert_eq!(dns_res.message_type(), DnsMessageType::Response);
    assert!(dns_res.id() > 0);
    assert_eq!(dns_res.query_count(), 1);
    assert_eq!(dns_res.answer_count(), 1);
    assert_eq!(dns_res.additional_count(), 0);
    assert_eq!(dns_res.name_server_count(), 0);

    assert_eq!(dns_res.queries().len(), 1);
    let dns_query = &(dns_res.queries())[0];
    assert_eq!(dns_query.query_type(), DnsRecordType::A);
    assert_eq!(dns_query.query_class(), DnsClass::IN);
    assert_eq!(dns_query.name().to_utf8(), "example.com.");

    assert_eq!(dns_res.answers().len(), 1);
    let dns_answer = &(dns_res.answers())[0];
    assert_eq!(dns_answer.record_type(), DnsRecordType::A);
    assert_eq!(dns_answer.dns_class(), DnsClass::IN);
    assert_eq!(dns_answer.name().to_utf8(), "example.com.");
    assert!(dns_answer.ttl() > 100);
    assert_eq!(dns_answer.rdata().to_record_type(), DnsRecordType::A);

    assert!(dns_res.edns().is_none());
  }

  #[test]
  fn should_resolve_udp_query_aaaa_www_ivandemarino_me() {
    let provider = DoHJsonProvider::available().remove(provider::PROVIDER_NAME_CLOUDFLARE).unwrap();

    let resolver = DoHJsonResolver::new(provider);

    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_AAAA-www.ivandemarino.me-packet.bin");
    let dns_req = DnsMessage::from_vec(&buf).unwrap();

    let dns_res_result = resolver.resolve_query(&dns_req);
    assert!(dns_res_result.is_ok());
    let dns_res = force_msg_finalization(dns_res_result.unwrap());

    assert_eq!(dns_res.message_type(), DnsMessageType::Response);
    assert!(dns_res.id() > 0);
    assert_eq!(dns_res.query_count(), 1);
    assert_eq!(dns_res.answer_count(), 2);
    assert_eq!(dns_res.additional_count(), 0);
    assert_eq!(dns_res.name_server_count(), 0);

    assert_eq!(dns_res.queries().len(), 1);
    let dns_query = &(dns_res.queries())[0];
    assert_eq!(dns_query.query_type(), DnsRecordType::AAAA);
    assert_eq!(dns_query.name().to_utf8(), "www.ivandemarino.me.");

    assert_eq!(dns_res.answers().len(), 2);

    let dns_answer = &(dns_res.answers())[0];
    assert_eq!(dns_answer.record_type(), DnsRecordType::CNAME);
    assert_eq!(dns_answer.name().to_utf8(), "www.ivandemarino.me.");
    assert!(dns_answer.ttl() > 100);
    match dns_answer.rdata() {
      DnsRData::CNAME(name) => assert_eq!(name.to_utf8(), "detro.github.com."),
      _ => panic!(),
    };

    let dns_answer = &(dns_res.answers())[1];
    assert_eq!(dns_answer.record_type(), DnsRecordType::CNAME);
    assert_eq!(dns_answer.name().to_utf8(), "detro.github.com.");
    assert!(dns_answer.ttl() > 100);
    match dns_answer.rdata() {
      DnsRData::CNAME(name) => assert_eq!(name.to_utf8(), "github.github.io."),
      _ => panic!(),
    };

    assert!(dns_res.edns().is_none());
  }
}