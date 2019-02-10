//! Implementation of `DoHResponse` for the DoH JSON Protocol.
//!
//! Based on [Serde JSON](https://crates.io/crates/serde_json).

pub use serde_json::Error as DoHParseError;

use core::response::DoHResponse;
use dns::protocol::*;
use serde_json::{self, Value};
use serde::{ser::Serializer, de::{Deserialize, Deserializer}};
use ipnet::IpNet;
use std::{str::FromStr, string::ToString, collections::HashMap, net::{Ipv4Addr, Ipv6Addr}};

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
  pub question: Vec<DoHJsonQuestion>,           //< See `DoHResponseQuestion` above
  #[serde(rename = "Answer")]
  pub answer: Vec<DoHJsonAnswer>,               //< See `DoHResponseAnswer` above
  #[serde(rename = "Additional", default)]
  pub additional: Vec<Value>,
  #[serde(default, serialize_with = "DoHJsonResponse::edns_client_subnet_serialize", deserialize_with = "DoHJsonResponse::edns_client_subnet_deserialize")]
  pub edns_client_subnet: Option<IpNet>,        //< IP address / scope prefix-length
  #[serde(rename = "Comment", default)]
  pub comment: String,

  // TODO Add support for "Authority" field (CloudFlare has it). Example:
  //   ".., "Authority":[{"name": "github.io.", "type": 6, "TTL": 60, "data": "ns1.p16.dynect.net. hostmaster.github.com. 92 3600 600 604800 60"}] }"
}

impl DoHJsonResponse {

  /// Serializes a `Option<IpNet>` into a `&str`
  ///
  /// Useful for Serde's `serialize_with` attribute
  fn edns_client_subnet_serialize<S>(edns_client_subnet: &Option<IpNet>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    match edns_client_subnet {
      Some(subnet) => serializer.serialize_str(format!("{}", subnet).as_ref()),
      None => serializer.serialize_str(String::default().as_ref()),
    }
  }

  /// Deserializes a `&str` into a `Option<IpNet>`
  ///
  /// Useful for Serde's `deserialize_with` attribute
  fn edns_client_subnet_deserialize<'de, D>(deserializer: D) -> Result<Option<IpNet>, D::Error> where D: Deserializer<'de> {
    let raw_edns_client_subnet: &str = Deserialize::deserialize(deserializer)?;
    match raw_edns_client_subnet.parse::<IpNet>() {
      Ok(subnet) => Ok(Some(subnet)),
      Err(_) => Ok(None)
    }
  }

  /// Dserializes a slice of bytes (`&[u8]`) into a `DoHJsonResponse`
  ///
  /// # Parameters
  ///
  /// * `bytes`: slice of bytes that can be deserialized to `DoHJsonResponse`
  pub fn from_slice(bytes: &[u8]) -> Result<Self, DoHParseError> {
    Ok(serde_json::from_slice(bytes)?)
  }

}

impl DoHResponse for DoHJsonResponse {

  fn apply(&self, req_edns_client_subnet_prefix_len: u8, res_dns_msg: &mut DnsMessage) {
    // Set control fields
    res_dns_msg.set_truncated(self.truncated);
    res_dns_msg.set_recursion_desired(self.recursion_desired);
    res_dns_msg.set_recursion_available(self.recursion_available);
    res_dns_msg.set_authentic_data(self.authenticated_data);
    res_dns_msg.set_checking_disabled(self.checking_disabled);
    res_dns_msg.set_response_code(self.response_code);

    // Add question fields
    for question in self.question.iter() {
      let q_name = DnsDomainName::from_str(question.name.as_ref()).unwrap();
      res_dns_msg.add_query(DnsQuery::query(q_name, question.question_type));
    }

    // Add answer fields
    for answer in self.answer.iter() {
      let r_name = DnsDomainName::from_str(answer.name.as_ref()).unwrap();
      let r_ttl = answer.ttl;
      let r_type = answer.answer_type;

      match r_type {
        DnsRecordType::A => {
          let r_ipv4addr = Ipv4Addr::from_str(answer.data.as_ref()).unwrap();
          res_dns_msg.add_answer(DnsRecord::from_rdata(r_name, r_ttl, r_type, DnsRData::A(r_ipv4addr)));
        },
        DnsRecordType::AAAA => {
          let r_ipv6addr = Ipv6Addr::from_str(answer.data.as_ref()).unwrap();
          res_dns_msg.add_answer(DnsRecord::from_rdata(r_name, r_ttl, r_type, DnsRData::AAAA(r_ipv6addr)));
        },
        DnsRecordType::CNAME => {
          let r_cname = DnsDomainName::from_str(answer.data.as_ref()).unwrap();
          res_dns_msg.add_answer(DnsRecord::from_rdata(r_name, r_ttl, r_type, DnsRData::CNAME(r_cname)));
        },
        DnsRecordType::NS => {
          let r_cname = DnsDomainName::from_str(answer.data.as_ref()).unwrap();
          res_dns_msg.add_answer(DnsRecord::from_rdata(r_name, r_ttl, r_type, DnsRData::NS(r_cname)));
        },
        DnsRecordType::PTR => {
          let r_cname = DnsDomainName::from_str(answer.data.as_ref()).unwrap();
          res_dns_msg.add_answer(DnsRecord::from_rdata(r_name, r_ttl, r_type, DnsRData::PTR(r_cname)));
        },
        _ => {
          error!("Unsupported DNS Answer Record type: {}", r_type);
        }
      };
    }

    // Add additional "EDNS Client Subnet OPT" if present
    if let Some(client_subnet) = self.edns_client_subnet {
      res_dns_msg.set_edns(edns_from_client_subnet(req_edns_client_subnet_prefix_len, &client_subnet));
    }
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
    serde_json::to_string(self).expect("Could not convert DoHJsonResponse to String")
  }
}

impl Default for DoHJsonResponse {
  /// Useful for testing
  fn default() -> Self {
    DoHJsonResponse {
      response_code: DnsResponseCode::NoError,
      truncated: false,
      recursion_desired: true,
      recursion_available: true,
      authenticated_data: false,
      checking_disabled: false,
      question: vec![],
      answer: vec![],
      additional: vec![],
      edns_client_subnet: Option::default(),
      comment: String::default()
    }
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

  const EXAMPLE_JSON_RESPONSE: &'static str = r#"{
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
    "edns_client_subnet": "12.34.56.0/0"
  }"#;

  #[test]
  fn should_deserialize_response() {
    let dns_resp = EXAMPLE_JSON_RESPONSE.parse::<DoHJsonResponse>().unwrap();

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
    assert_eq!(dns_resp.edns_client_subnet, Some("12.34.56.0/0".parse::<IpNet>().unwrap()));
    assert_eq!(dns_resp.comment, "");

    println!("{:#?}", dns_resp);
  }

  #[test]
  fn should_serialize_response() {
    let dns_resp_json_orig = r#"{"Status":0,"TC":false,"RD":true,"RA":true,"AD":false,"CD":false,"Question":[{"name":"apple.com.","type":1}],"Answer":[{"name":"apple.com.","type":1,"TTL":3599,"data":"17.178.96.59"},{"name":"apple.com.","type":1,"TTL":3599,"data":"17.172.224.47"},{"name":"apple.com.","type":1,"TTL":3599,"data":"17.142.160.59"}],"Additional":[],"edns_client_subnet":"12.34.56.78/0","Comment":""}"#;
    let dns_resp: DoHJsonResponse = dns_resp_json_orig.parse().unwrap();
    assert_eq!(dns_resp.to_string(), dns_resp_json_orig);

    let dns_resp: DoHJsonResponse = DoHJsonResponse::default();
    assert_eq!(dns_resp.edns_client_subnet, None);
    assert_eq!(dns_resp.to_string(), r#"{"Status":0,"TC":false,"RD":true,"RA":true,"AD":false,"CD":false,"Question":[],"Answer":[],"Additional":[],"edns_client_subnet":"","Comment":""}"#);
  }

  #[test]
  fn should_apply_to_dns_message() {
    let mut dns_msg = DnsMessage::new();

    // DnsMessage just created
    assert_eq!(dns_msg.truncated(), false);
    assert_eq!(dns_msg.recursion_desired(), false);
    assert_eq!(dns_msg.recursion_available(), false);
    assert_eq!(dns_msg.authentic_data(), false);
    assert_eq!(dns_msg.checking_disabled(), false);
    assert_eq!(dns_msg.response_code(), DnsResponseCode::NoError);
    assert!(dns_msg.edns().is_none());
    assert_eq!(dns_msg.query_count(), 0u16);
    assert_eq!(dns_msg.answer_count(), 0u16);
    assert_eq!(dns_msg.name_server_count(), 0u16);
    assert_eq!(dns_msg.additional_count(), 0u16);

    let dns_resp = EXAMPLE_JSON_RESPONSE.parse::<DoHJsonResponse>().unwrap();
    dns_resp.apply(10, &mut dns_msg);

    // Check DNS Message control
    assert_eq!(dns_msg.truncated(), false);
    assert_eq!(dns_msg.recursion_desired(), true);
    assert_eq!(dns_msg.recursion_available(), true);
    assert_eq!(dns_msg.authentic_data(), false);
    assert_eq!(dns_msg.checking_disabled(), false);
    assert_eq!(dns_msg.response_code(), DnsResponseCode::NoError);

    // Check queries (there should be just 1)
    assert_eq!(dns_msg.queries().len(), 1);
    let q = &dns_msg.queries()[0];
    assert_eq!(q.name().to_utf8(), "apple.com.");
    assert_eq!(q.query_type(), DnsRecordType::A);

    // Check answers (there should be 3)
    assert_eq!(dns_msg.answers().len(), 3);
    let a = &dns_msg.answers()[0];
    assert_eq!(a.name().to_utf8(), "apple.com.");
    assert_eq!(a.record_type(), DnsRecordType::A);
    assert_eq!(a.ttl(), 3599);
    assert_eq!(a.rdata(), &DnsRData::A(Ipv4Addr::new(17, 178, 96, 59)));
    let a = &dns_msg.answers()[1];
    assert_eq!(a.name().to_utf8(), "apple.com.");
    assert_eq!(a.record_type(), DnsRecordType::A);
    assert_eq!(a.ttl(), 3599);
    assert_eq!(a.rdata(), &DnsRData::A(Ipv4Addr::new(17, 172, 224, 47)));
    let a = &dns_msg.answers()[2];
    assert_eq!(a.name().to_utf8(), "apple.com.");
    assert_eq!(a.record_type(), DnsRecordType::A);
    assert_eq!(a.ttl(), 3599);
    assert_eq!(a.rdata(), &DnsRData::A(Ipv4Addr::new(17, 142, 160, 59)));

    // Check EDNS
    assert!(dns_msg.edns().is_some());
    assert_eq!(dns_msg.edns().unwrap().options().options().len(), 1);
    assert!(dns_msg.edns().unwrap().option(&DnsRDataOPTCode::Subnet).is_some());
    let edns_opt = dns_msg.edns().unwrap().option(&DnsRDataOPTCode::Subnet).unwrap();
    match edns_opt {
      DnsRDataOPTOption::Unknown(len, data) => {
        assert_eq!(data.len(), *len as usize);
        assert_eq!(&data[..], [0, 1, 10, 0, 12, 34, 56, 0] as [u8; 8]);
      },
      _ => {
        panic!("This should never happen!");
      }
    };

    assert_eq!(dns_msg.name_servers().len(), 0);
    assert_eq!(dns_msg.additionals().len(), 0);
  }

}
