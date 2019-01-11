use doh_json::{response::*, provider::DoHJsonProvider};
use core::{provider::*, resolver::*, response::*};
use dns::protocol::{DnsMessage, DnsMessageType};

type Result<T> = std::result::Result<T, DoHResolutionError>;

struct DoHJsonResolver {
  provider: DoHJsonProvider
}

impl DoHResolver for DoHJsonResolver {

  fn resolve_message_query(&self, req_dns_msg: DnsMessage) -> Result<DnsMessage> {
    // Begin preparing response DNS Message
    let mut res_dns_msg = DnsMessage::new();
    res_dns_msg.set_id(req_dns_msg.id());
    res_dns_msg.set_op_code(req_dns_msg.op_code());
    res_dns_msg.set_message_type(DnsMessageType::Response);

    // Execute all queries in parallel
    for query in req_dns_msg.queries() {
      let req = self.provider.build_http_request(query)?;

      // TODO
      //  1. execute request
      //  2. parse response body into DoHJsonResponse
      //  3. DoHJsonResponse.apply(res_dns_msg)
    }

    Ok(res_dns_msg)
  }

}