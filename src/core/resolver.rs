//! `DnsMessage` resolution

use dns::protocol::{DnsMessage, DnsMessageType};
use http::Error as HttpError;

use std::{fmt, error};

type Result<T> = std::result::Result<T, ResolutionError>;

/// A type of `Error` emitted by `Resolver`
///
/// It contains a description and an optional `HttpError` that might have caused it
#[derive(Debug)]
pub struct ResolutionError {
  desc: &'static str,
  src: Option<HttpError>,
}

impl fmt::Display for ResolutionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ResolutionError: {}", self.desc)
  }
}

impl error::Error for ResolutionError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match &self.src {
      Some(src) => Some(src),
      None => None
    }
  }
}

/// A _resolver_ of `DnsMessage` requests
pub trait Resolver {

  /// Resolves a DNS Query and returns a DNS Response
  ///
  /// It assumes the input `DnsMessage` is of type `DnsMessageType::Query`:
  /// it's strongly adviced that client code uses the `Resolver::resolve()` method instead,
  /// as it takes care of doing this crucial check before attempting the resolution.
  ///
  /// # Parameters
  ///
  /// * `dns_message` - A `DnsMessage` that we assume is of type `DnsMessageType::Query`
  fn resolve_message_query(&self, dns_message: DnsMessage) -> Result<DnsMessage>;

  /// Resolves a DNS Query and returns a DNS Response
  ///
  /// It checks that the `DnsMessage` is of type `DnsMessageType::Query`: if not, it
  /// throws an error of type `ResolutionError`.
  ///
  /// The actual resolution is then delegated to the specific implementation of
  /// `Resolver::resolve_message_query()`.
  ///
  /// # Parameters
  ///
  /// * `dns_message` - A `DnsMessage` that we assume is of type `DnsMessageType::Query`
  fn resolve(&self, dns_message: DnsMessage) -> Result<DnsMessage> {
    // Before resolving, check the type is right
    if dns_message.message_type() == DnsMessageType::Query {
      self.resolve_message_query(dns_message)
    } else {
      Err(ResolutionError {
        desc: "Invalid input: `DnsMessage` was not of type `Query`",
        src: None
      })
    }
  }

}