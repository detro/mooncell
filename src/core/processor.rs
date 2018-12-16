use std::sync::mpsc::Receiver;

use dns_proto::DnsMessage;
use net::request_responder::DnsRequestResponder;
use doh::protocol::DoHResponse;

pub struct Processor {
  receiver: Receiver<DnsRequestResponder>,
}