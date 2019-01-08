use std::sync::mpsc::Receiver;

use net::request_responder::DnsRequestResponder;

#[allow(dead_code)]
pub struct Processor {
  receiver: Receiver<DnsRequestResponder>,
}