use hyper::{Method, Request, Body};
use serde_json::{self, Error as SerdeJsonError, Value};
use std::{collections::HashMap, str::FromStr, string::ToString};
use url::Url;

// ----------------------------------------------------------------------------- DoHResponseQuestion
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHResponseQuestion {
  name: String,
  #[serde(rename = "type", default = "DoHResponseQuestion::question_type_default")]
  question_type: u16,
}

impl DoHResponseQuestion {
  fn question_type_default() -> u16 {
    1
  }
}

// ------------------------------------------------------------------------------- DoHResponseAnswer
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHResponseAnswer {
  name: String,
  #[serde(rename = "type", default = "DoHResponseAnswer::answer_type_default")]
  answer_type: u16,
  #[serde(rename = "TTL")]
  ttl: u32,
  data: String,
}

impl DoHResponseAnswer {
  fn answer_type_default() -> u16 {
    1
  }
}

// ------------------------------------------------------------------------------------- DoHResponse
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHResponse {
  #[serde(rename = "Status")]
  status: u32,
  #[serde(rename = "TC")]
  truncated: bool,
  #[serde(rename = "RD")]
  recursion_desired: bool,
  #[serde(rename = "RA")]
  recursion_available: bool,
  #[serde(rename = "AD")]
  authenticated_data: bool,
  #[serde(rename = "CD")]
  checking_disabled: bool,
  #[serde(rename = "Question")]
  question: Vec<DoHResponseQuestion>,
  #[serde(rename = "Answer")]
  answer: Vec<DoHResponseAnswer>,
  #[serde(rename = "Additional", default)]
  additional: Vec<Value>,
  #[serde(default)]
  edns_client_subnet: String,
  #[serde(rename = "Comment", default)]
  comment: String,
}

impl FromStr for DoHResponse {
  type Err = SerdeJsonError;

  fn from_str(doh_response_json: &str) -> Result<Self, SerdeJsonError> {
    Ok(serde_json::from_str(doh_response_json)?)
  }
}

// ------------------------------------------------------------------------------------- DoHProvider
pub struct DoHProvider<'a> {
  method: Method,
  host: &'a str,
  path: &'a str,
  mandatory_query: Vec<(&'a str, &'a str)>,
}

impl<'a> DoHProvider<'a> {
  pub const DEFAULT_KEY_GOOGLE: &'static str = "google";
  pub const DEFAULT_KEY_CLOUDFLARE: &'static str = "cloudflare";

  pub fn defaults<'b>() -> HashMap<&'b str, DoHProvider<'b>> {
    let mut providers = HashMap::new();

    providers.insert(Self::DEFAULT_KEY_GOOGLE, DoHProvider {
      method: Method::GET,
      host: "dns.google.com",
      path: "resolve",
      mandatory_query: vec![],
    });
    providers.insert(Self::DEFAULT_KEY_CLOUDFLARE, DoHProvider {
      method: Method::GET,
      host: "cloudflare-dns.com",
      path: "dns-query",
      mandatory_query: vec![("ct", "application/dns-json")],
    });

    providers
  }

  pub fn url(&self) -> Url {
    let mut url = Url::parse(&format!("https://{}", self.host)).unwrap();
    url.set_path(self.path);

    for query_param in self.mandatory_query.iter() {
      url.query_pairs_mut().append_pair(query_param.0, query_param.1);
    }

    url
  }
}

// -------------------------------------------------------------------------------- Request building
pub fn build_request<'a>(provider: &'a DoHProvider, query_name: &str, query_type: &str) -> Request<Body> {
  // Append to the URL required by the Provider, the query parameters
  let mut url = provider.url();
  url.query_pairs_mut()
    .append_pair("name", query_name)
    .append_pair("type", query_type);

  // Assemble the final request
  Request::builder()
    .method(provider.method.clone())
    .uri(url.to_string())
    .body(Body::empty())
    .unwrap()
}

// ------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod test {
  use super::*;
  use hyper::Version;

  fn should_deserialize_response() {
    let dns_resp_json = r#"{
    "Status": 0,
    "TC": false,
    "RD": true,
    "RA": true,
    "AD": false,
    "CD": false,
    "Question": [
      {
        "name": "apple.com.",
        "type": 1
      }
    ],
    "Answer": [
      {
        "name": "apple.com.",
        "type": 1,
        "TTL": 3599,
        "data": "17.178.96.59"
      },
      {
        "name": "apple.com.",
        "type": 1,
        "TTL": 3599,
        "data": "17.172.224.47"
      },
      {
        "name": "apple.com.",
        "type": 1,
        "TTL": 3599,
        "data": "17.142.160.59"
      }
    ],
    "Additional": [ ],
    "edns_client_subnet": "12.34.56.78/0"
  }"#;

    let dns_resp = DoHResponse::from_str(dns_resp_json).unwrap();

    assert_eq!(dns_resp.status, 0);
    assert_eq!(dns_resp.truncated, false);
    assert_eq!(dns_resp.recursion_desired, true);
    assert_eq!(dns_resp.recursion_available, true);
    assert_eq!(dns_resp.authenticated_data, false);
    assert_eq!(dns_resp.checking_disabled, false);

    assert_eq!(dns_resp.question.len(), 1);
    assert_eq!(dns_resp.question.get(0).unwrap().name, "apple.com.");
    assert_eq!(dns_resp.question.get(0).unwrap().question_type, 1);

    assert_eq!(dns_resp.answer.len(), 3);
    for answer in &(dns_resp.answer) {
      assert_eq!(answer.name, "apple.com.");
      assert_eq!(answer.answer_type, 1);
      assert_eq!(answer.ttl, 3599);
      assert!(answer.data.starts_with("17."));
    }

    assert_eq!(dns_resp.additional.len(), 0);
    assert_eq!(dns_resp.edns_client_subnet, "12.34.56.78/0");
    assert_eq!(dns_resp.comment, "");
  }

  #[test]
  fn should_serialize_response() {
    let dns_resp_json_orig = r#"{"Status":0,"TC":false,"RD":true,"RA":true,"AD":false,"CD":false,"Question":[{"name":"apple.com.","type":1}],"Answer":[{"name":"apple.com.","type":1,"TTL":3599,"data":"17.178.96.59"},{"name":"apple.com.","type":1,"TTL":3599,"data":"17.172.224.47"},{"name":"apple.com.","type":1,"TTL":3599,"data":"17.142.160.59"}],"Additional":[],"edns_client_subnet":"12.34.56.78/0","Comment":""}"#;

    let dns_resp = DoHResponse::from_str(dns_resp_json_orig).unwrap();

    assert_eq!(serde_json::to_string(&dns_resp).unwrap(), dns_resp_json_orig);
  }

  #[test]
  fn should_produce_default_providers() {
    let def_providers = DoHProvider::defaults();

    let google_provider = def_providers.get(DoHProvider::DEFAULT_KEY_GOOGLE).unwrap();
    assert_eq!(google_provider.host, "dns.google.com");
    assert_eq!(google_provider.path, "resolve");
    assert_eq!(google_provider.mandatory_query.len(), 0);
    assert_eq!(google_provider.url().to_string(), "https://dns.google.com/resolve");

    let cloudflare_provider = def_providers.get(DoHProvider::DEFAULT_KEY_CLOUDFLARE).unwrap();
    assert_eq!(cloudflare_provider.host, "cloudflare-dns.com");
    assert_eq!(cloudflare_provider.path, "dns-query");
    assert_eq!(cloudflare_provider.mandatory_query.len(), 1);
    assert_eq!(cloudflare_provider.mandatory_query[0].0, "ct");
    assert_eq!(cloudflare_provider.mandatory_query[0].1, "application/dns-json");
    assert_eq!(cloudflare_provider.url().to_string(), "https://cloudflare-dns.com/dns-query?ct=application%2Fdns-json");
  }

  #[test]
  fn should_create_request() {
    let default_providers = DoHProvider::defaults();

    let request = build_request(default_providers.get(DoHProvider::DEFAULT_KEY_GOOGLE).unwrap(), "newrelic", "A");
    assert_eq!(request.version(), Version::HTTP_11);
    assert_eq!(request.method(), &Method::GET);
    assert_eq!(request.uri(), "https://dns.google.com/resolve?name=newrelic&type=A");
    assert_eq!(request.body(), &());
    assert!(request.headers().is_empty());

    let request = build_request(default_providers.get(DoHProvider::DEFAULT_KEY_CLOUDFLARE).unwrap(), "newrelic", "A");
    assert_eq!(request.version(), Version::HTTP_11);
    assert_eq!(request.method(), &Method::GET);
    assert_eq!(request.uri(), "https://cloudflare-dns.com/dns-query?ct=application%2Fdns-json&name=newrelic&type=A");
    assert_eq!(request.body(), &());
    assert!(request.headers().is_empty());
  }
}
