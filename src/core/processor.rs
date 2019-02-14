//! Processor...

use std::sync::mpsc::Receiver;
use net::request::Request;
use core::resolver::DoHResolver;

pub struct Processor {
  receiver: Receiver<Request>,
  resolver: Box<dyn DoHResolver>,
}

impl Processor {
  pub fn new(receiver: Receiver<Request>, resolver: Box<dyn DoHResolver>) -> Processor {
    Processor {
      receiver,
      resolver
    }
  }
}