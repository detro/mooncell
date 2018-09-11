use doh::protocol::{build_request, DoHProvider, DoHResponse};
use hyper::{Body, Client, client::HttpConnector};
use std::{collections::HashMap, io::Result, str::from_utf8, str::FromStr};
use tokio::{self, prelude::*};
use trust_dns_proto::op::{Header, MessageType, OpCode, ResponseCode};
use trust_dns_server::authority::MessageResponse;
use trust_dns_server::server::{Request, RequestHandler, ResponseHandler};

pub struct DnsQueryHandler<'a> {
  default_providers: HashMap<&'a str, DoHProvider<'a>>,
  http_client: Client<HttpConnector, Body>,
}

impl<'a> DnsQueryHandler<'a> {
  pub fn new() -> Self {
    DnsQueryHandler {
      default_providers: DoHProvider::defaults(),
      http_client: Client::builder().keep_alive(true).build_http(),
    }
  }

  fn configured_provider(&self) -> &DoHProvider {
    self.default_providers.get(DoHProvider::DEFAULT_KEY_GOOGLE).unwrap()
  }
}

impl<'a> RequestHandler for DnsQueryHandler<'a> {
  fn handle_request<R: ResponseHandler + 'static>(&self,
                                                  request: &Request,
                                                  response_handle: R) -> Result<()> {
    debug!("Request src: {:#?}", request.src);
    debug!("Request message: {:#?}", request.message);
    debug!("Request queries: {}", request.message.queries().len());

    // Submit a DoH query per DNS Query
    for query in request.message.queries() {
      let q_type = query.query_type();
      let q_name = &format!("{}", query.name());

      let doh_request = build_request(self.configured_provider(), q_name, q_type.into());
      debug!("DoH {:#?}", doh_request);

      let doh_request_processing_future = self.http_client.request(doh_request)
        .and_then(|res| {
          trace!("Response status: {}", res.status());
          res.into_body().concat2()
        })
        .map(|_| {
          info!("All done");
        })
//        .and_then(|body| {
//          let json_response = from_utf8(&body)
//            .expect("Response should be JSON");
//          let doh_respose = DoHResponse::from_str(json_response)
//            .expect("Not a valid DoH response");
//
////          let mut response = MessageResponse::new(None); //< TODO: provide the actual query that got this response
////          let mut response_header = Header::new();
////          response_header.set_id(request.message.id());
////          response_header.set_op_code(OpCode::Query);
////          response_header.set_message_type(MessageType::Response);
////          response_header.set_response_code(ResponseCode::NoError);
////          response_header.set_authoritative(true);
////          response.answers(records.unwrap());
////          response.name_servers(ns.unwrap());
////          response.build(response_header)
//        });
        .map_err(|err| {
          error!("Query failed: {}", err);
        });


      tokio::spawn(doh_request_processing_future);
    }


    // TODO Execute a request using 'tokio::spawn'
    // TODO Gather response body
    // TODO Deserialize JSON response using 'serde::json'
    // TODO Assemble responsne message
    // TODO Return response message

    Ok(())
  }
}
