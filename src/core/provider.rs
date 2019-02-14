//! Trait definition for DNS-over-HTTPS provider

use core::protocol::DoHProtocol;
use downcast_rs::Downcast;
use dns::protocol::DnsQuery;
use http::{Result, Request};
use std::{collections::HashMap, fmt};

/// Trait defining a provider of DNS-over-HTTPS services
pub trait DoHProvider: Downcast {

  /// Provider identifier
  fn id(&self) -> &str;

  /// Protocol supported by the Provider
  fn protocol(&self) -> DoHProtocol;

  /// Builds an HTTP request combining the information of the `DoHProvider` with the given `DnsQuery`
  ///
  /// This is the important part of this type: taking a "standard" `DnsQuery` and turning it into
  /// an actual HTTP request that we can send to the give `DoHProvider` and, hopefully, get
  /// a DNS resolution back.
  ///
  /// # Parameters
  ///
  /// * `dns_query` - `DnsQuery` that we need to turn into an HTTP request towards the Provider
  fn build_http_request(&self, dns_query: &DnsQuery) -> Result<Request<()>>;

  /// Available Providers of DoH services
  ///
  /// Providers are organised in a `HashMap`, indexed by identifier (of type `&'static str`).
  /// This allows to _pick_ them programmatically when needed.
  fn available() -> HashMap<&'static str, Self> where Self: Sized;

  /// Default provider identifier
  fn default_id() -> &'static str where Self: Sized;

  /// Vector of Identifiers of available Providers
  fn available_ids() -> Vec<&'static str> where Self: Sized {
    Self::available().iter().map(|(key, _)| *key).collect()
  }

  /// Default provider
  fn default() -> Self where Self: Sized {
    Self::available()
      .remove(Self::default_id())
      .expect("There should always be a Provider associated to default ID: this should never happen!")
  }

}

impl fmt::Debug for DoHProvider {
  fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
    write!(fmtr, "DoHProvider: {{ protocol: {}, id: {} }}", self.protocol(), self.id())
  }
}

impl_downcast!(DoHProvider);