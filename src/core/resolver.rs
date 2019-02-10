//! Trait definition for resolver of `DnsMessage` requests via DNS-over-HTTPS

use dns::protocol::{DnsMessage, DnsMessageType};
use http::Error as HttpError;
use curl::Error as CurlError;
use serde_json::Error as SerdeJsonError;
use std::{fmt, convert};

type Result<T> = std::result::Result<T, DoHResolutionError>;

/// A type of `Error` emitted by `Resolver`
///
/// It contains a description and an optional `HttpError` that might have caused it
#[derive(Debug)]
pub struct DoHResolutionError {
  desc: String,
}

impl DoHResolutionError {
  pub fn new(desc: String) -> DoHResolutionError {
    DoHResolutionError { desc }
  }
}

impl fmt::Display for DoHResolutionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ResolutionError: {}", self.desc)
  }
}

impl convert::From<HttpError> for DoHResolutionError {
  fn from(http_error: HttpError) -> Self {
    DoHResolutionError {
      desc: format!("Failed to execute HTTP request (http): {}", http_error),
    }
  }
}

impl convert::From<SerdeJsonError> for DoHResolutionError {
  fn from(serde_json_error: SerdeJsonError) -> Self {
    DoHResolutionError {
      desc: format!("Failed to parse JSON (serde_json): {}", serde_json_error),
    }
  }
}

impl convert::From<CurlError> for DoHResolutionError {
  fn from(curl_error: CurlError) -> Self {
    DoHResolutionError {
      desc: format!("Failed to execute HTTP request (cURL): {}", curl_error),
    }
  }
}

/// Trait defining a _resolver_ of `DnsMessage` queries
pub trait DoHResolver {

  /// Resolves a DNS Query and returns a DNS Response
  ///
  /// It assumes the input `DnsMessage` is of type `DnsMessageType::Query`:
  /// it's strongly advised that client code uses the `Resolver::resolve()` method instead,
  /// as it takes care of doing this crucial check before attempting the resolution.
  ///
  /// # Parameters
  ///
  /// * `dns_message` - A `DnsMessage` that we assume is of type `DnsMessageType::Query`
  fn resolve_query(&self, dns_message: &DnsMessage) -> Result<DnsMessage>;

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
  fn resolve(&self, dns_message: &DnsMessage) -> Result<DnsMessage> {
    // Before resolving, check the type is right
    if dns_message.message_type() == DnsMessageType::Query {
      self.resolve_query(dns_message)
    } else {
      Err(DoHResolutionError::new("Invalid input: `DnsMessage` was not of type `Query`".into()))
    }
  }

}