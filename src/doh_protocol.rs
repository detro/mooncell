use serde_json::{self, Error, Value};

#[derive(Serialize, Deserialize)]
pub struct DnsQuestion {
  name: String,
  #[serde(rename = "type", default = "DnsQuestion::question_type_default")]
  question_type: u16,
}

impl DnsQuestion {
  fn question_type_default() -> u16 {
    1
  }
}

#[derive(Serialize, Deserialize)]
pub struct DnsAnswer {
  name: String,
  #[serde(rename = "type", default = "DnsAnswer::answer_type_default")]
  answer_type: u16,
  #[serde(rename = "TTL")]
  ttl: u32,
  data: String,
}

impl DnsAnswer {
  fn answer_type_default() -> u16 {
    1
  }
}

#[derive(Serialize, Deserialize)]
pub struct DnsResponse {
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
  question: Vec<DnsQuestion>,
  #[serde(rename = "Answer")]
  answer: Vec<DnsAnswer>,
  #[serde(rename = "Additional")]
  additional: Vec<Value>,
  edns_client_subnet: String,
  #[serde(rename = "Comment", default)]
  comment: String,
}

#[test]
fn dns_response_type_should_deserialize_correctly() {
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

  let dns_resp: DnsResponse = serde_json::from_str(dns_resp_json).unwrap();

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
fn dns_response_type_should_serialize_correctly() {
  let dns_resp_json_orig = r#"{"Status":0,"TC":false,"RD":true,"RA":true,"AD":false,"CD":false,"Question":[{"name":"apple.com.","type":1}],"Answer":[{"name":"apple.com.","type":1,"TTL":3599,"data":"17.178.96.59"},{"name":"apple.com.","type":1,"TTL":3599,"data":"17.172.224.47"},{"name":"apple.com.","type":1,"TTL":3599,"data":"17.142.160.59"}],"Additional":[],"edns_client_subnet":"12.34.56.78/0","Comment":""}"#;

  let dns_resp: DnsResponse = serde_json::from_str(dns_resp_json_orig).unwrap();

  assert_eq!(serde_json::to_string(&dns_resp).unwrap(), dns_resp_json_orig);
}
