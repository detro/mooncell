//! Enum of possible DNS-over-HTTPS protocols

use std::{fmt, convert::From};

const PROTOCOL_NAME_JSON: &'static str = "json";
const PROTOCOL_NAME_WIRE: &'static str = "wire";

/// DNS-over-HTTPS protocol
pub enum DoHProtocol {
  JSON,
  WIRE
}

impl From<DoHProtocol> for &'static str {
  fn from(protocol: DoHProtocol) -> Self {
    match protocol {
      DoHProtocol::JSON => PROTOCOL_NAME_JSON,
      DoHProtocol::WIRE => PROTOCOL_NAME_WIRE,
    }
  }
}

impl From<&str> for DoHProtocol {
  fn from(raw_protocol: &str) -> Self {
    match raw_protocol {
      PROTOCOL_NAME_JSON => DoHProtocol::JSON,
      PROTOCOL_NAME_WIRE => DoHProtocol::WIRE,
      _ => panic!("Invalid DoH Protocol: {}", raw_protocol)
    }
  }
}

impl fmt::Display for DoHProtocol {
  fn fmt(&self, fmtr: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      DoHProtocol::JSON => write!(fmtr, "{}", PROTOCOL_NAME_JSON),
      DoHProtocol::WIRE => write!(fmtr, "{}", PROTOCOL_NAME_WIRE),
    }
  }
}