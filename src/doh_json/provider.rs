//! DoH JSON provider(s)

use crate::core::{provider::DoHProvider, protocol::DoHProtocol};
use crate::dns::protocol::*;

use http::{
  method::Method,
  version::Version,
  uri::{Builder as UriBuilder, Scheme, Authority, PathAndQuery},
  header::{HeaderMap, self},
  request::{Request, Builder as RequestBuilder},
  Result,
};

use std::{collections::HashMap, str::FromStr};

// TODO Add support for optional parameters: hopefully Google and the others have compatible,
//  optional parameters

/// Describes a provider of DNS-over-HTTPS services
#[derive(Debug, Clone)]
pub struct DoHJsonProvider {
  id: &'static str,
  scheme: Scheme,
  authority: Authority,
  path_query: PathAndQuery,
  headers: HeaderMap,
}

impl DoHJsonProvider {
  /// Constructor from "raw" parts
  ///
  /// # Parameters
  ///
  /// * `id` - Identifier of this Provider
  /// * `raw_scheme` - `&str` representing the scheme of a URI (ex. "http", "https" or others)
  /// * `raw_authority` - `&str` representing the authority of a URI (ex. "example.com" or "other-example.com:8081")
  /// * `raw_path_query` - `&str` representing the path and query of a URI (ex. "/path/to/file?q1=v1&q2=v2")
  fn from_raw_parts(id: &'static str, raw_scheme: &str, raw_authority: &str, raw_path_query: &str) -> DoHJsonProvider {
    DoHJsonProvider::from_parts(
      id,
      raw_scheme.parse().unwrap(),
      raw_authority.parse().unwrap(),
      raw_path_query.parse().unwrap(),
      HeaderMap::default()
    )
  }

  /// Constructor from "raw" parts that allows to provide headers
  ///
  /// # Parameters
  ///
  /// * `id` - Identifier of this Provider
  /// * `raw_scheme` - `&str` representing the scheme of a URI (ex. "http", "https" or others)
  /// * `raw_authority` - `&str` representing the authority of a URI (ex. "example.com" or "other-example.com:8081")
  /// * `raw_path_query` - `&str` representing the path and query of a URI (ex. "/path/to/file?q1=v1&q2=v2")
  /// * `headers` - an `HeaderMap` as defined by the `http` crate
  fn from_raw_parts_with_headers(id: &'static str, raw_scheme: &str, raw_authority: &str, raw_path_query: &str, headers: HeaderMap) -> DoHJsonProvider {
    DoHJsonProvider::from_parts(
      id,
      raw_scheme.parse().unwrap(),
      raw_authority.parse().unwrap(),
      raw_path_query.parse().unwrap(),
      headers
    )
  }

  /// Constructor from parts
  ///
  /// # Parameters
  ///
  /// * `id` - Identifier of this Provider
  /// * `scheme` - `Scheme` of a URI (ex. "http", "https" or others)
  /// * `authority` - `Authority` of a URI (ex. "example.com" or "other-example.com:8081")
  /// * `path_query` - `PathAndQuery` of a URI (ex. "/path/to/file?q1=v1&q2=v2")
  /// * `headers` - an `HeaderMap` as defined by the `http` crate
  fn from_parts(id: &'static str, scheme: Scheme, authority: Authority, path_query: PathAndQuery, headers: HeaderMap) -> DoHJsonProvider {
    DoHJsonProvider {
      id,
      scheme,
      authority,
      path_query,
      headers
    }
  }
}

/// Static `&str` identifier for [Google Public DNS-over-HTTPS](https://developers.google.com/speed/public-dns/docs/dns-over-https) provider
pub const PROVIDER_NAME_GOOGLE: &'static str = "google";
/// Static `&str` identifier for [Cloudflare DNS-over-HTTPS](https://developers.cloudflare.com/1.1.1.1/dns-over-https/json-format/) provider
pub const PROVIDER_NAME_CLOUDFLARE: &'static str = "cloudflare";
/// Static `&str` identifier for [Quad9 DNS-over-HTTPS](https://www.quad9.net/doh-quad9-dns-servers/) "Recommended" provider
pub const PROVIDER_NAME_QUAD9: &'static str = "quad9";
/// Static `&str` identifier for [Quad9 DNS-over-HTTPS](https://www.quad9.net/doh-quad9-dns-servers/) "Secured" provider
pub const PROVIDER_NAME_QUAD9_SECURED: &'static str = "quad9-secured";
/// Static `&str` identifier for [Quad9 DNS-over-HTTPS](https://www.quad9.net/doh-quad9-dns-servers/) "Unsecured" provider
pub const PROVIDER_NAME_QUAD9_UNSECURED: &'static str = "quad9-unsecured";
/// Static `&str` identifier for [Rubyfish DNS-over-HTTPS](https://www.rubyfish.cn/dns-query) provider (preferable in China)
pub const PROVIDER_NAME_RUBYFISH: &'static str = "rubyfish";
/// Static `&str` identifier for [BlahDNS DNS-over-HTTPS](https://blahdns.com/) provider (Preferable in Japan)
pub const PROVIDER_NAME_BLAHDNS:  &'static str = "blahdns";

impl DoHProvider for DoHJsonProvider {

  fn id(&self) -> &'static str {
    self.id
  }

  fn protocol(&self) -> DoHProtocol {
    DoHProtocol::JSON
  }

  fn build_http_request(&self, dns_query: &DnsQuery) -> Result<Request<()>> {
    // Prepare Path and Query parts of the request, combining the Provider "required" parts
    // with the actual DNS Query
    let query_type: &str = dns_query.query_type().into();
    let query_name = dns_query.name().to_string();
    let path_query = if let Some(provider_required_query) = self.path_query.query() {
      PathAndQuery::from_str(&format!("{}?type={}&name={}&{}", self.path_query.path(), query_type, query_name, provider_required_query))?
    } else {
      PathAndQuery::from_str(&format!("{}?type={}&name={}", self.path_query.path(), query_type, query_name))?
    };

    // Compose the request URI by assembling all it's parts
    let uri = UriBuilder::new()
      .scheme(self.scheme.clone())
      .authority(self.authority.clone())
      .path_and_query(path_query)
      .build()?;

    // Using a Request builder to assemble the final HTTP Request
    let mut req_builder = RequestBuilder::new();
    // Adding some defaults as well as URI
    req_builder
      .version(Version::HTTP_11)
      .method(Method::GET)
      .uri(uri);

    // Adding extra headers (if any)
    for (hkey, hval) in self.headers.iter() {
      req_builder.header(hkey, hval);
    }

    req_builder.body(())
  }

  fn available() -> HashMap<&'static str, Self> {
    let mut providers = HashMap::new();

    // Google
    providers.insert(PROVIDER_NAME_GOOGLE, DoHJsonProvider::from_raw_parts(
      PROVIDER_NAME_GOOGLE,
      "https",
      "dns.google.com",
      "/resolve"
    ));
    // Cloudflare
    let mut cloudflare_headers = HeaderMap::with_capacity(1);
    cloudflare_headers.insert(header::ACCEPT, "application/dns-json".parse().unwrap());
    providers.insert(PROVIDER_NAME_CLOUDFLARE, DoHJsonProvider::from_raw_parts_with_headers(
      PROVIDER_NAME_CLOUDFLARE,
      "https",
      "cloudflare-dns.com",
      "/dns-query",
      cloudflare_headers
    ));
    // Quad9 recommended
    providers.insert(PROVIDER_NAME_QUAD9, DoHJsonProvider::from_raw_parts(
      PROVIDER_NAME_QUAD9,
      "https",
      "dns.quad9.net",
      "/dns-query"
    ));
    // Quad9 secured
    providers.insert(PROVIDER_NAME_QUAD9_SECURED, DoHJsonProvider::from_raw_parts(
      PROVIDER_NAME_QUAD9_SECURED,
      "https",
      "dns9.quad9.net",
      "/dns-query"
    ));
    // Quad9 unsecured
    providers.insert(PROVIDER_NAME_QUAD9_UNSECURED, DoHJsonProvider::from_raw_parts(
      PROVIDER_NAME_QUAD9_UNSECURED,
      "https",
      "dns10.quad9.net",
      "/dns-query"
    ));
    // Rubyfish
    providers.insert(PROVIDER_NAME_RUBYFISH, DoHJsonProvider::from_raw_parts(
      PROVIDER_NAME_RUBYFISH,
      "https",
      "dns.rubyfish.cn",
      "/dns-query"
    ));
    // BlahDNS
    providers.insert(PROVIDER_NAME_BLAHDNS, DoHJsonProvider::from_raw_parts(
      PROVIDER_NAME_BLAHDNS,
      "https",
      "doh-de.blahdns.com",
      "/dns-query"
    ));

    providers
  }

  fn default_id() -> &'static str {
    PROVIDER_NAME_CLOUDFLARE
  }

}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn should_provide_cloudflare_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("ivandemarino.me.").unwrap(), DnsRecordType::AAAA);

    let default_provider: DoHJsonProvider = DoHJsonProvider::default();
    let cloudflare_provider = DoHJsonProvider::available().get(PROVIDER_NAME_CLOUDFLARE).unwrap().to_owned();

    // What's going on here? We are testing the same thing twice, as Cloudflare is also the default provider
    let providers = vec![default_provider, cloudflare_provider];

    for provider in providers {
      let http_request = provider.build_http_request(&example_query).unwrap();
      assert_eq!(http_request.method(), Method::GET);
      assert_eq!(http_request.version(), Version::HTTP_11);
      assert_eq!(http_request.uri().to_string(), "https://cloudflare-dns.com/dns-query?type=AAAA&name=ivandemarino.me.");
      assert_eq!(http_request.extensions().get::<bool>(), None);
      assert!(http_request.headers().contains_key(header::ACCEPT));
      assert_eq!(http_request.headers().get(header::ACCEPT).unwrap(), &"application/dns-json");
      assert_eq!(http_request.headers().len(), 1);
      assert_eq!(http_request.body(), &());
    }
  }

  #[test]
  fn should_provide_google_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("github.com.").unwrap(), DnsRecordType::A);
    let default_providers = DoHJsonProvider::available();

    let provider = default_providers.get(PROVIDER_NAME_GOOGLE).unwrap();

    let http_request = provider.build_http_request(&example_query).unwrap();
    assert_eq!(http_request.method(), Method::GET);
    assert_eq!(http_request.version(), Version::HTTP_11);
    assert_eq!(http_request.uri().to_string(), "https://dns.google.com/resolve?type=A&name=github.com.");
    assert_eq!(http_request.extensions().get::<bool>(), None);
    assert_eq!(http_request.headers().len(), 0);
    assert_eq!(http_request.body(), &());
  }

  #[test]
  fn should_provide_quad9_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("github.com.").unwrap(), DnsRecordType::A);
    let default_providers = DoHJsonProvider::available();

    let provider = default_providers.get(PROVIDER_NAME_QUAD9).unwrap();

    let http_request = provider.build_http_request(&example_query).unwrap();
    assert_eq!(http_request.method(), Method::GET);
    assert_eq!(http_request.version(), Version::HTTP_11);
    assert_eq!(http_request.uri().to_string(), "https://dns.quad9.net/dns-query?type=A&name=github.com.");
    assert_eq!(http_request.extensions().get::<bool>(), None);
    assert_eq!(http_request.headers().len(), 0);
    assert_eq!(http_request.body(), &());
  }

  #[test]
  fn should_provide_quad9_secured_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("github.com.").unwrap(), DnsRecordType::A);
    let default_providers = DoHJsonProvider::available();

    let provider = default_providers.get(PROVIDER_NAME_QUAD9_SECURED).unwrap();

    let http_request = provider.build_http_request(&example_query).unwrap();
    assert_eq!(http_request.method(), Method::GET);
    assert_eq!(http_request.version(), Version::HTTP_11);
    assert_eq!(http_request.uri().to_string(), "https://dns9.quad9.net/dns-query?type=A&name=github.com.");
    assert_eq!(http_request.extensions().get::<bool>(), None);
    assert_eq!(http_request.headers().len(), 0);
    assert_eq!(http_request.body(), &());
  }

  #[test]
  fn should_provide_quad9_unsecured_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("github.com.").unwrap(), DnsRecordType::A);
    let default_providers = DoHJsonProvider::available();

    let provider = default_providers.get(PROVIDER_NAME_QUAD9_UNSECURED).unwrap();

    let http_request = provider.build_http_request(&example_query).unwrap();
    assert_eq!(http_request.method(), Method::GET);
    assert_eq!(http_request.version(), Version::HTTP_11);
    assert_eq!(http_request.uri().to_string(), "https://dns10.quad9.net/dns-query?type=A&name=github.com.");
    assert_eq!(http_request.extensions().get::<bool>(), None);
    assert_eq!(http_request.headers().len(), 0);
    assert_eq!(http_request.body(), &());
  }

  #[test]
  fn should_provide_rubyfish_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("apple.com.").unwrap(), DnsRecordType::A);
    let default_providers = DoHJsonProvider::available();

    let provider = default_providers.get(PROVIDER_NAME_RUBYFISH).unwrap();

    let http_request = provider.build_http_request(&example_query).unwrap();
    assert_eq!(http_request.method(), Method::GET);
    assert_eq!(http_request.version(), Version::HTTP_11);
    assert_eq!(http_request.uri().to_string(), "https://dns.rubyfish.cn/dns-query?type=A&name=apple.com.");
    assert_eq!(http_request.extensions().get::<bool>(), None);
    assert_eq!(http_request.headers().len(), 0);
    assert_eq!(http_request.body(), &());
  }

  #[test]
  fn should_provide_blahdns_provider() {
    let example_query = DnsQuery::query(DnsDomainName::from_str("apple.com.").unwrap(), DnsRecordType::A);
    let default_providers = DoHJsonProvider::available();

    let provider = default_providers.get(PROVIDER_NAME_BLAHDNS).unwrap();

    let http_request = provider.build_http_request(&example_query).unwrap();
    assert_eq!(http_request.method(), Method::GET);
    assert_eq!(http_request.version(), Version::HTTP_11);
    assert_eq!(http_request.uri().to_string(), "https://doh-de.blahdns.com/dns-query?type=A&name=apple.com.");
    assert_eq!(http_request.extensions().get::<bool>(), None);
    assert_eq!(http_request.headers().len(), 0);
    assert_eq!(http_request.body(), &());
  }

  #[test]
  fn should_provide_default_provider_ids() {
    assert_eq!(DoHJsonProvider::available_ids().len(), 7);
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_CLOUDFLARE));
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_GOOGLE));
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_QUAD9));
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_QUAD9_SECURED));
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_QUAD9_UNSECURED));
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_RUBYFISH));
    assert!(DoHJsonProvider::available_ids().contains(&PROVIDER_NAME_BLAHDNS));
  }

}