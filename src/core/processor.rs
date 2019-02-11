//! Processor...

use std::sync::mpsc::Receiver;
use net::request::Request;

pub struct Processor {
  receiver: Receiver<Request>,
}