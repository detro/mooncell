//! Trait definition for DNS-over-HTTPS provider

use dns::protocol::DnsQuery;
use http::{Result, Request};
use std::{collections::HashMap, default::Default};

/// Trait defining a provider of DNS-over-HTTPS services
pub trait DoHProvider: Default {

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

  /// Default providers of DoH services
  ///
  /// Providers are organised in a `HashMap`, indexed by identifier (of type `&'static str`_.
  /// This allows to _pick_ them programmatically when needed.
  fn defaults() -> HashMap<&'static str, Self>;

  /// Vector of default providers identifiers
  fn default_ids() -> Vec<&'static str>;

}