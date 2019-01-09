use crate::doh_json::{protocol::*,provider::DoHJsonProvider};

use core::{provider::*, resolver::*};
use dns::protocol::DnsMessage;

type Result<T> = std::result::Result<T, ResolutionError>;

struct DoHJsonResolver {
  provider: DoHJsonProvider
}

impl DoHResolver for DoHJsonResolver {
  fn resolve_message_query(&self, dns_message: DnsMessage) -> Result<DnsMessage> {
    let msg_id = dns_message.id();

    for query in dns_message.queries() {
      let req = self.provider.build_http_request(query)?;

      // TODO
      //  1. execute request
      //  2. parse response body into DoHJsonResponse
      //  3. DoHJsonResponse -> DnsMessage (implement an `Into<DnsMessage>`?)
    }

    unimplemented!()
  }
}