use std::io::Result;
use trust_dns_server::server::{Request, RequestHandler, ResponseHandler};

pub struct DnsQueryHandler {}

impl DnsQueryHandler {
  pub fn new() -> Self {
    DnsQueryHandler {}
  }
}

impl RequestHandler for DnsQueryHandler {

  fn handle_request<'q, 'a, R: ResponseHandler + 'static>(
    &'a self,
    request: &'q Request,
    response_handle: R,
  ) -> Result<()> {
    info!("Request src: {:?}", request.src);
    info!("Request message: {:?}", request.message);

    // TODO Extract the queries from message
    // TODO Assemble a request using 'doh::protocol'
    // TODO Execute a request using 'tokio::spawn'
    // TODO Gather response body
    // TODO Deserialize JSON response using 'serde::json'
    // TODO Assemble responsne message
    // TODO Return response message

    Ok(())
  }

}

//    let request_message = &request.message;
//    let request_src = &request.src;
//    debug!("Received request from: {}", request_src.ip().to_string());
//    debug!("ID: {}", request_message.id());
//    debug!("Queries: ");
//    request_message.queries().iter().for_each(|q| {
//      debug!("  {}", q);
//    });
//    debug!("Authoritative: {}", request_message.authoritative());
//    debug!("Type: {:?}", request_message.message_type());
//    debug!("Op code: {:?}", request_message.op_code());
//    debug!("Recursion desired: {}", request_message.recursion_desired());
//    debug!("Recursion available: {}", request_message.recursion_available());
//
//    // @see https://developers.cloudflare.com/1.1.1.1/dns-over-https/json-format/
//    // https://cloudflare-dns.com/dns-query?
//    //    ?ct=application/dns-json
//    //    &name=newrelic.com
//    //    &type=A
//    //
//    // @see https://developers.google.com/speed/public-dns/docs/dns-over-https#dns_response_in_json
//    // https://dns.google.com/resolve
//    //    ?name=newrelic.com
//    //    &type=A
//    //    &cd=1
//    //    &edns_client_subnet=0.0.0.0/0
//
//    let url = Url::parse_with_params("https://cloudflare-dns.com/dns-query", &[("name", "newrelic.com"), ("type", "A"), ("ct", "application/dns-json")]).unwrap();
//    let resp_json = reqwest::get(url).unwrap().text().unwrap();
//    debug!("------");
//    debug!("{}", resp_json);
//    debug!("------");
//
//    let mut response = Message::new();
//    response.set_message_type(MessageType::Response);
//    response.set_op_code(OpCode::Query);
//    response.set_response_code(ResponseCode::NoError);
//    response.set_id(request_message.id());
//
//    let mut response_record = Record::new();
//    response_record.set_name(Name::from_str("www.google.com.").unwrap());
//    response_record.set_rdata(RData::A(Ipv4Addr::from_str("55.44.33.22").unwrap()));
//    response.add_answer(response_record);
//
//    let mut response_record = Record::new();
//    response_record.set_name(Name::from_str("www.google.com.").unwrap());
//    response_record.set_rdata(RData::A(Ipv4Addr::from_str("55.44.33.23").unwrap()));
//    response.add_answer(response_record);
//
//    response
//  }
//}
