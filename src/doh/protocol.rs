pub use serde_json::Error as DoHResponseParseError;

use dns::protocol::*;
use serde_json::{self, Value};
use std::{str::FromStr, string::ToString};

/// Represents the deserialized response body for an DNS-over-HTTPS request
///
/// The JSON Schema is **not** standardised but major players have decided to play friendly
/// and keep it consistent. A couple of examples:
///
/// * [Cloudflare DNS-over-HTTPS](https://developers.cloudflare.com/1.1.1.1/dns-over-https/json-format/)
/// * [Google DNS-over-HTTPS](https://developers.google.com/speed/public-dns/docs/dns-over-https)
///
/// Hopefully, eventually, there will be a standard document to refer to.
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHResponse {
  #[serde(rename = "Status", serialize_with = "dns_response_code_serialize", deserialize_with = "dns_response_code_deserialize")]
  pub response_code: DnsResponseCode,
  #[serde(rename = "TC")]
  pub truncated: bool,                          //< Whether the response is truncated
  #[serde(rename = "RD")]
  pub recursion_desired: bool,                  //< Recursion desired
  #[serde(rename = "RA")]
  pub recursion_available: bool,                //< Recursion available
  #[serde(rename = "AD")]
  pub authenticated_data: bool,                 //< Whether all response data was validated with DNSSEC
  #[serde(rename = "CD")]
  pub checking_disabled: bool,                  //< Whether the client asked to disable DNSSEC
  #[serde(rename = "Question")]
  pub question: Vec<DoHResponseQuestion>,       //< See `DoHResponseQuestion` above
  #[serde(rename = "Answer")]
  pub answer: Vec<DoHResponseAnswer>,           //< See `DoHResponseAnswer` above
  #[serde(rename = "Additional", default)]
  pub additional: Vec<Value>,
  #[serde(default)]
  pub edns_client_subnet: String,               //< IP address / scope prefix-length
  #[serde(rename = "Comment", default)]
  pub comment: String,
}

impl FromStr for DoHResponse {
  type Err = DoHResponseParseError;

  fn from_str(doh_response_json: &str) -> Result<Self, Self::Err> {
    Ok(serde_json::from_str(doh_response_json)?)
  }
}

impl ToString for DoHResponse {
  fn to_string(&self) -> String {
    serde_json::to_string(self).expect("Could not convert DoHResponse to String")
  }
}

/// Question part of a `DoHResponse` type
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHResponseQuestion {
  pub name: String,                             //< FQDN with trailing dot
  #[serde(rename = "type", default = "DoHResponseQuestion::question_type_default", serialize_with = "dns_record_type_serialize", deserialize_with = "dns_record_type_deserialize")]
  pub question_type: DnsRecordType,             //< Standard DNS RR type (default "A")
}

impl DoHResponseQuestion {
  fn question_type_default() -> DnsRecordType {
    DnsRecordType::A
  }
}

/// Answer part of a `DoHResponse` type
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHResponseAnswer {
  pub name: String,                             //< FQDN with trailing dot
  #[serde(rename = "type", default = "DoHResponseAnswer::answer_type_default", serialize_with = "dns_record_type_serialize", deserialize_with = "dns_record_type_deserialize")]
  pub answer_type: DnsRecordType,               //< Standard DNS RR type (default "A")
  #[serde(rename = "TTL")]
  pub ttl: u32,                                 //< Record's time-to-live in seconds
  pub data: String,                             //< Data for A - IP address as text
}

impl DoHResponseAnswer {
  fn answer_type_default() -> DnsRecordType {
    DnsRecordType::A
  }
}

#[cfg(test)]
mod test {
  use super::*;

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

    let dns_resp = dns_resp_json.parse::<DoHResponse>().unwrap();

    assert_eq!(dns_resp.response_code, DnsResponseCode::NoError);
    assert_eq!(dns_resp.truncated, false);
    assert_eq!(dns_resp.recursion_desired, true);
    assert_eq!(dns_resp.recursion_available, true);
    assert_eq!(dns_resp.authenticated_data, false);
    assert_eq!(dns_resp.checking_disabled, false);

    assert_eq!(dns_resp.question.len(), 1);
    assert_eq!(dns_resp.question.get(0).unwrap().name, "apple.com.");
    assert_eq!(dns_resp.question.get(0).unwrap().question_type, DnsRecordType::A);

    assert_eq!(dns_resp.answer.len(), 3);
    for answer in &(dns_resp.answer) {
      assert_eq!(answer.name, "apple.com.");
      assert_eq!(answer.answer_type, DnsRecordType::A);
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

    let dns_resp: DoHResponse = dns_resp_json_orig.parse().unwrap();

    assert_eq!(dns_resp.to_string(), dns_resp_json_orig);
  }

}
