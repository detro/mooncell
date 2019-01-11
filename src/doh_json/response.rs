//! Representation of types used by DoH JSON Protocol.
//!
//! Based on [Serde JSON](https://crates.io/crates/serde_json).

pub use serde_json::Error as DoHParseError;

use core::response::DoHResponse;
use dns::protocol::*;
use serde_json::{self, Value};
use ipnet::IpNet;
use std::{str::FromStr, string::ToString, collections::HashMap};

/// Represents the deserialized response body for a DNS-over-HTTPS JSON request
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHJsonResponse {
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
  pub question: Vec<DoHJsonQuestion>,       //< See `DoHResponseQuestion` above
  #[serde(rename = "Answer")]
  pub answer: Vec<DoHJsonAnswer>,           //< See `DoHResponseAnswer` above
  #[serde(rename = "Additional", default)]
  pub additional: Vec<Value>,
  #[serde(default)]
  pub edns_client_subnet: String,               //< IP address / scope prefix-length
  #[serde(rename = "Comment", default)]
  pub comment: String,
}

impl DoHResponse for DoHJsonResponse {

  fn apply(&self, req_edns_client_subnet_prefix_len: u8, res_dns_msg: &mut DnsMessage) {
    res_dns_msg.set_authoritative(self.authenticated_data);
    res_dns_msg.set_truncated(self.truncated);
    res_dns_msg.set_recursion_desired(self.recursion_desired);
    res_dns_msg.set_recursion_available(self.recursion_available);
    res_dns_msg.set_authentic_data(self.authenticated_data);
    res_dns_msg.set_checking_disabled(self.checking_disabled);
    res_dns_msg.set_response_code(self.response_code);

    if let Ok(client_subnet) = self.edns_client_subnet.parse::<IpNet>() {
      res_dns_msg.set_edns(edns_from_client_subnet(req_edns_client_subnet_prefix_len, &client_subnet));
    }

    // TODO Add Answers
    // TODO Add Queries too?
  }

}

impl FromStr for DoHJsonResponse {
  type Err = DoHParseError;

  fn from_str(doh_response_json: &str) -> Result<Self, Self::Err> {
    Ok(serde_json::from_str(doh_response_json)?)
  }
}

impl ToString for DoHJsonResponse {
  fn to_string(&self) -> String {
    serde_json::to_string(self).expect("Could not convert DoHResponse to String")
  }
}

/// Create a `DnsEdns` struct from a response's `edns_client_subnet` `String`
///
/// See [official documentation](https://tools.ietf.org/html/rfc7871#page-4) for the format
/// of EDNS Client Subnet OPT RR format.
///
/// # Parameters
///
/// * `req_subnet_prefix_len`: subnet size as requested by the client in the request
/// * `res_subnet`: `&IpNet` representing the EDNS Client Subnet received in a `DoHJsonResponse`
fn edns_from_client_subnet(req_subnet_prefix_len: u8, res_subnet: &IpNet) -> DnsEdns {
  // Assemble the RDATA for OPT Record
  let option_code = DnsRDataOPTCode::Subnet;
  let option_data: Vec<u8> = match res_subnet {
    IpNet::V4(subnet_ipv4) => {
      let mut data = Vec::with_capacity(8);

      // Family (see IANA Address Family Numbers: https://www.iana.org/assignments/address-family-numbers/address-family-numbers.xhtml)
      data.extend_from_slice(&[0u8, 1u8]);            //< 2 octets for family
      // Source prefix
      data.push(req_subnet_prefix_len);
      // Scope prefix
      data.push(subnet_ipv4.prefix_len());
      // Address
      data.extend_from_slice(&subnet_ipv4.addr().octets());  //< 4 octets in a IPv4 address

      data
    },
    IpNet::V6(subnet_ipv6) => {
      let mut data = Vec::with_capacity(20);

      // Family (see IANA Address Family Numbers: https://www.iana.org/assignments/address-family-numbers/address-family-numbers.xhtml)
      data.extend_from_slice(&[0u8, 2u8]);            //< 2 octets for family
      // Source prefix
      data.push(req_subnet_prefix_len);
      // Scope prefix
      data.push(subnet_ipv6.prefix_len());
      // Address
      data.extend_from_slice(&subnet_ipv6.addr().octets());  //< 16 octets in a IPv6 address

      data
    }
  };
  let option_length = option_data.len() as u16; //< `option_data` can't exceed `u16` by design of the protocol

  let mut rdata_options: HashMap<DnsRDataOPTCode, DnsRDataOPTOption> = HashMap::with_capacity(1);
  rdata_options.insert(option_code, DnsRDataOPTOption::Unknown(
    option_length,
    option_data
  ));

  let ttl = 0u32;
  let rdata = DnsRData::OPT(DnsRDataOPT::new(rdata_options));
  let opt_record = DnsRecord::from_rdata(DnsDomainName::root(), ttl, DnsRecordType::OPT, rdata);

  // Generate EDNS from OPT Record
  DnsEdns::from(&opt_record)
}

/// Question part of a `DoHResponse` type
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHJsonQuestion {
  pub name: String,                             //< FQDN with trailing dot
  #[serde(rename = "type", default = "DoHJsonQuestion::question_type_default", serialize_with = "dns_record_type_serialize", deserialize_with = "dns_record_type_deserialize")]
  pub question_type: DnsRecordType,             //< Standard DNS RR type (default "A")
}

impl DoHJsonQuestion {
  fn question_type_default() -> DnsRecordType {
    DnsRecordType::A
  }
}

/// Answer part of a `DoHResponse` type
#[derive(Serialize, Deserialize, Debug)]
pub struct DoHJsonAnswer {
  pub name: String,                             //< FQDN with trailing dot
  #[serde(rename = "type", default = "DoHJsonAnswer::answer_type_default", serialize_with = "dns_record_type_serialize", deserialize_with = "dns_record_type_deserialize")]
  pub answer_type: DnsRecordType,               //< Standard DNS RR type (default "A")
  #[serde(rename = "TTL")]
  pub ttl: u32,                                 //< Record's time-to-live in seconds
  pub data: String,                             //< Data for A - IP address as text
}

impl DoHJsonAnswer {
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

    let dns_resp = dns_resp_json.parse::<DoHJsonResponse>().unwrap();

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

    let dns_resp: DoHJsonResponse = dns_resp_json_orig.parse().unwrap();

    assert_eq!(dns_resp.to_string(), dns_resp_json_orig);
  }

}
