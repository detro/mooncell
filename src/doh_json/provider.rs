//! DoH JSON provider(s)

use dns::protocol::DnsMessage;

use http::{
  method::Method,
  uri::{Builder as UriBuilder, Scheme, Authority, PathAndQuery},
  request::{Request, Builder as RequestBuilder},
  Result as HttpResult,
};

use std::collections::HashMap;

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

lazy_static! {
  /// Providers of DoH JSON services
  ///
  /// They are organised in an `HashMap` so we can programmatically list and/or select them.
  /// Please look at the constant static `&str` prefixed with "`PROVIDER_NAME_`",
  /// present in this module, to find what keys exist in this map by default.
  pub static ref DEFAULT_PROVIDERS: HashMap<&'static str, DoHProvider> = {
    let mut providers = HashMap::new();

    // Google
    providers.insert(PROVIDER_NAME_GOOGLE, DoHProvider::from_raw_parts(
      "https",
      "dns.google.com",
      "/resolve"
    ));
    // Cloudflare
    providers.insert(PROVIDER_NAME_CLOUDFLARE, DoHProvider::from_raw_parts(
      "https",
      "cloudflare-dns.com",
      "/dns-query"
    ));
    // Quad9 recommended
    providers.insert(PROVIDER_NAME_QUAD9, DoHProvider::from_raw_parts(
      "https",
      "dns.quad9.net",
      "/dns-query"
    ));
    // Quad9 secured
    providers.insert(PROVIDER_NAME_QUAD9_SECURED, DoHProvider::from_raw_parts(
      "https",
      "dns9.quad9.net",
      "/dns-query"
    ));
    // Quad9 unsecured
    providers.insert(PROVIDER_NAME_QUAD9_UNSECURED, DoHProvider::from_raw_parts(
      "https",
      "dns10.quad9.net",
      "/dns-query"
    ));
    // Rubyfish
    providers.insert(PROVIDER_NAME_RUBYFISH, DoHProvider::from_raw_parts(
      "https",
      "dns.rubyfish.cn",
      "/dns-query"
    ));
    // BlahDNS
    providers.insert(PROVIDER_NAME_BLAHDNS, DoHProvider::from_raw_parts(
      "https",
      "doh-de.blahdns.com",
      "/dns-query"
    ));

    providers
  };
}

// TODO Add support for optional parameters: hopefully Google and the others have compatible,
//  optional parameters

/// A provider of DNS-over-HTTPS services
pub struct DoHProvider {
  scheme: Scheme,
  authority: Authority,
  path_query: PathAndQuery,
  // TODO Add mandatory headers like `accept: application/dns-json` for Cloudflare
}

impl DoHProvider {

  pub fn from_raw_parts(raw_scheme: &str, raw_authority: &str, raw_path_query: &str) -> DoHProvider {
    DoHProvider::from_parts(
      raw_scheme.parse().unwrap(),
      raw_authority.parse().unwrap(),
      raw_path_query.parse().unwrap()
    )
  }

  pub fn from_parts(scheme: Scheme, authority: Authority, path_query: PathAndQuery) -> DoHProvider {
    DoHProvider {
      scheme,
      authority,
      path_query,
    }
  }

  pub fn build_request(&mut self, dns_msg: &DnsMessage) -> HttpResult<Request<()>> {
    let uri = UriBuilder::new()
      .scheme(self.scheme.clone())
      .authority(self.authority.clone())
      // TODO Add query parameters 'type' and 'name' and any other required to describe `dns_msg`
      .path_and_query(self.path_query.clone())
      .build()?;

    RequestBuilder::new()
      .method(Method::GET)
      .uri(uri)
      .body(())
  }

}

// -------------------------------------------------------------------------------- Request building
//pub fn build_request<'a>(provider: &'a DoHProvider, query_name: &str, query_type: &str) -> Request<Body> {
//  // Append to the URL required by the Provider, the query parameters
//  let mut url = provider.url();
//  url.query_pairs_mut()
//    .append_pair("name", query_name)
//    .append_pair("type", query_type);
//
//  // Assemble the final request
//  Request::builder()
//    .method(provider.method.clone())
//    .uri(url.to_string())
//    .body(Body::empty())
//    .unwrap()
//}

// ------------------------------------------------------------------------------------------- Tests
//#[cfg(test)]
//mod test {
//  use super::*;
//  use http::Version;
//
//  #[test]
//  fn should_produce_default_providers() {
//    let def_providers = DoHProvider::defaults();
//
//    let google_provider = def_providers.get(DoHProvider::DEFAULT_KEY_GOOGLE).unwrap();
//    assert_eq!(google_provider.host, "dns.google.com");
//    assert_eq!(google_provider.path, "resolve");
//    assert_eq!(google_provider.mandatory_query.len(), 0);
//    assert_eq!(google_provider.url().to_string(), "https://dns.google.com/resolve");
//
//    let cloudflare_provider = def_providers.get(DoHProvider::DEFAULT_KEY_CLOUDFLARE).unwrap();
//    assert_eq!(cloudflare_provider.host, "cloudflare-dns.com");
//    assert_eq!(cloudflare_provider.path, "dns-query");
//    assert_eq!(cloudflare_provider.mandatory_query.len(), 1);
//    assert_eq!(cloudflare_provider.mandatory_query[0].0, "ct");
//    assert_eq!(cloudflare_provider.mandatory_query[0].1, "application/dns-json");
//    assert_eq!(cloudflare_provider.url().to_string(), "https://cloudflare-dns.com/dns-query?ct=application%2Fdns-json");
//  }
//
//  #[test]
//  fn should_create_request() {
//    let default_providers = DoHProvider::defaults();
//
//    let request = build_request(default_providers.get(DoHProvider::DEFAULT_KEY_GOOGLE).unwrap(), "newrelic", "A");
//    assert_eq!(request.version(), Version::HTTP_11);
//    assert_eq!(request.method(), &Method::GET);
//    assert_eq!(request.uri(), "https://dns.google.com/resolve?name=newrelic&type=A");
//    assert_eq!(request.body(), &());
//    assert!(request.headers().is_empty());
//
//    let request = build_request(default_providers.get(DoHProvider::DEFAULT_KEY_CLOUDFLARE).unwrap(), "newrelic", "A");
//    assert_eq!(request.version(), Version::HTTP_11);
//    assert_eq!(request.method(), &Method::GET);
//    assert_eq!(request.uri(), "https://cloudflare-dns.com/dns-query?ct=application%2Fdns-json&name=newrelic&type=A");
//    assert_eq!(request.body(), &());
//    assert!(request.headers().is_empty());
//  }
//}