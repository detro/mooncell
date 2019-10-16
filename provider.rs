//! DNS-over-HTTPS service Provider

// standard
use std::{collections::{HashMap, HashSet}, fmt, net::IpAddr};

// local
use crate::{
  core::protocol::DoHProtocol,
  dns::protocol::DnsQuery,
};

// third-party
use http::{
  method::Method,
  version::Version,
  uri::{Builder as UriBuilder, Scheme, Authority, PathAndQuery},
  header::{HeaderMap, self},
  request::{Request, Builder as RequestBuilder},
  Result,
};
use url::Url;

/// A Provider of DNS-over-HTTPS services
///
/// It defines the required URL and Headers content for the HTTP request to be sent to the provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoHProvider {
  id: &'static str,
  protocols: HashSet<DoHProtocol>,
  base_url: Url,
  headers: HeaderMap,
}

/// Trait defining a provider of DNS-over-HTTPS services
impl DoHProvider {

  /// New DNS-over-HTTPS Provider
  fn new(id: &'static str, protocols: HashSet<DoHProtocol>, raw_base_url: &str) -> DoHProvider {
    DoHProvider::new_with_headers(id, protocols, raw_base_url, HeaderMap::default())
  }

  /// New DNS-over-HTTPS Provider, with required HTTP Headers
  fn new_with_headers(id: &'static str, protocols: HashSet<DoHProtocol>, raw_base_url: &str, mut headers: HeaderMap) -> DoHProvider {
    let base_url = Url::parse(raw_base_url).unwrap();
    headers.insert(header::HOST, base_url.host_str().unwrap().parse());

    DoHProvider {
      id,
      protocols,
      base_url,
      headers,
    }
  }

  /// Provider identifier
  fn id(&self) -> &str {
    self.id
  }

  /// DNS-over-HTTPS Protocols supported by the Provider
  fn protocols(&self) -> &HashSet<DoHProtocol> {
    &self.protocols
  }

  /// Execute a one-off "classic" DNS lookup, so to find the IP of the Provider, based on the known Authority
  fn resolve_authority(&mut self) {
    // TODO
  }

  /// Has this Provider's Authority IP been resolved?
  fn is_authority_resolved(&self) -> bool {
    self.base_url.has_host() && self.base_url.domain().is_none()
  }

  /// Provider "base" URL to be augmented with the DNS Query before sending the HTTP Request
  fn base_url(&self) -> &Url {
    &self.base_url
  }

  /// Provider Headers necessary for the HTTP Request to be executed
  fn headers(&self) -> &HeaderMap {
    &self.headers
  }

}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn should_describe_a_doh_provider() {

  }

}