//! Enum of possible DNS-over-HTTPS protocols

use std::{fmt, convert::From, str::FromStr};

const PROTOCOL_NAME_JSON: &'static str = "json";
const PROTOCOL_NAME_WIRE: &'static str = "wire";

/// DNS-over-HTTPS protocol
#[derive(Debug, Copy, Clone)]
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
    match DoHProtocol::from_str(raw_protocol) {
      Ok(protocol) => protocol,
      Err(err) => panic!(err)
    }
  }
}

impl FromStr for DoHProtocol {
  type Err = DoHProtocolParseError;

  fn from_str(raw_protocol: &str) -> Result<Self, Self::Err> {
    match raw_protocol {
      PROTOCOL_NAME_JSON => Ok(DoHProtocol::JSON),
      PROTOCOL_NAME_WIRE => Ok(DoHProtocol::WIRE),
      _ => Err(DoHProtocolParseError::new(raw_protocol))
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

/// Error that happens when parsing a `DoHProtocol` fails
#[derive(Debug, Clone)]
pub struct DoHProtocolParseError {
  token: String
}

impl DoHProtocolParseError {
  fn new(token: &str) -> Self {
    Self {
      token: token.to_string()
    }
  }
}

impl fmt::Display for DoHProtocolParseError {
  fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
    write!(fmtr, "Invalid DNS-over-HTTPS Protocol: {}", self.token)
  }
}
