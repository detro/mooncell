//! Processor...

use std::sync::mpsc::Receiver;
use net::request::Request;
use core::resolver::DoHResolver;

pub struct Processor {
  receiver: Receiver<Request>,
  resolver: DoHResolver,
}

impl Processor {}