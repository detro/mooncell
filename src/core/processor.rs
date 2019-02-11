//! Processor...

use std::sync::mpsc::Receiver;
use net::request_responder::DnsRequest;

pub struct Processor {
  receiver: Receiver<DnsRequest>,
}