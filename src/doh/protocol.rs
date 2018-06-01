use reqwest::{Method, Client, Url, Request, Error};

use serde_json::{self, Error as SerdeJsonError, Value};

use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;

// ----------------------------------------------------------------------------- DoHResponseQuestion
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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
  #[serde(rename = "Additional")]
  additional: Vec<Value>,
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
pub struct DoHProvider {
  method: Method,
  base_url: Url,
  mandatory_params: Vec<(String, String)>
}

pub const DEFAULT_DOH_PROVIDER_GOOGLE: &'static str = "google";
pub const DEFAULT_DOH_PROVIDER_CLOUDFLARE: &'static str = "cloudflare";

pub fn default_providers<'a>() -> HashMap<&'a str, DoHProvider> {
  let mut providers = HashMap::new();

  providers.insert(DEFAULT_DOH_PROVIDER_GOOGLE, DoHProvider {
    method: Method::Get,
    base_url: Url::parse("https://dns.google.com/resolve").unwrap(),
    mandatory_params: vec![]
  });
  providers.insert(DEFAULT_DOH_PROVIDER_CLOUDFLARE, DoHProvider {
    method: Method::Get,
    base_url: Url::parse("https://cloudflare-dns.com/dns-query").unwrap(),
    mandatory_params: vec![("ct".to_string(), "application/dns-json".to_string())]
  });

  providers
}

// -------------------------------------------------------------------------------- Reqwest building
const QUERY_PARAM_NAME: &'static str = "name";
const QUERY_PARAM_TYPE: &'static str = "type";

pub fn build_reqwest(client: &Client, provider: &DoHProvider, query_name: &str, query_type: &str) -> Result<Request, Error> {
  // Create "Reqwest" object using the provider "method" and "base url"
  client.request(provider.method.clone(), provider.base_url.clone())
    // Add the "mandatory query parameters" of the specific provider
    .query(provider.mandatory_params.as_slice())
    // Add the query "name" and "type"
    .query(&[(QUERY_PARAM_NAME, query_name), (QUERY_PARAM_TYPE, query_type)])
    .build()
}

// ------------------------------------------------------------------------------------------- Tests
#[test]
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
fn should_create_request() {
  let client = Client::new();
  let default_providers = default_providers();

  let request = build_reqwest(&client, &default_providers.get(DEFAULT_DOH_PROVIDER_GOOGLE).unwrap(), "newrelic", "A").unwrap();
  assert_eq!(request.method(), &Method::Get);
  assert_eq!(request.url().as_str(), "https://dns.google.com/resolve?name=newrelic&type=A");

  let request = build_reqwest(&client, &default_providers.get(DEFAULT_DOH_PROVIDER_CLOUDFLARE).unwrap(), "newrelic", "A").unwrap();
  assert_eq!(request.method(), &Method::Get);
  assert_eq!(request.url().as_str(), "https://cloudflare-dns.com/dns-query?ct=application%2Fdns-json&name=newrelic&type=A");
}
