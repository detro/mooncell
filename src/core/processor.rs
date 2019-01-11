//! Processor...

use std::sync::mpsc::Receiver;
use net::request_responder::DnsRequestResponder;

pub struct Processor {
  receiver: Receiver<DnsRequestResponder>,
}