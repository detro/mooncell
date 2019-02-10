//! Module that re-publishes the part of Trust-DNS Proto that we care about, and adds some utilities
//! to avoid having to use `trust_dns_proto` in other modules.
//!
//! Additionally, it takes care of the "consequences" of the [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits)
//! in regards of **serde**: to be able to use some `trust_dns_proto` types for serialization and
//! deserialization, a [special workaround](https://serde.rs/remote-derive.html) is needed.

pub use trust_dns_proto::error::{
  ProtoError as DnsProtoError,
  ProtoErrorKind as DnsProtoErrorKind,
};
pub use trust_dns_proto::{
  op::{
    header::MessageType as DnsMessageType,
    message::Message as DnsMessage,
    query::Query as DnsQuery,
    op_code::OpCode as DnsOpCode,
    response_code::ResponseCode as DnsResponseCode,
    Edns as DnsEdns,
  },
  rr::{
    record_type::RecordType as DnsRecordType,
    dnssec::rdata::DNSSECRecordType as DnsDNSSECRecordType,
    dns_class::DNSClass as DnsClass,
    domain::Name as DnsDomainName,
    resource::Record as DnsRecord,
    record_data::RData as DnsRData,
    rdata::opt::{OPT as DnsRDataOPT, EdnsCode as DnsRDataOPTCode, EdnsOption as DnsRDataOPTOption},
  },
};

use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};
use serde::{ser::Serializer, de::{Deserialize, Deserializer}};

/// Converts an array of raw bytes into a `DnsMessage`
pub fn dns_message_from_bytes(bytes: &[u8]) -> Result<DnsMessage, DnsProtoError> {
  DnsMessage::from_bytes(bytes)
}

/// Converts a `DnsMessage` to a vector of raw bytes
pub fn dns_message_to_bytes(message: &DnsMessage) -> Result<Vec<u8>, DnsProtoError> {
  message.to_bytes()
}

/// Serializes a `DnsResponseCode` into a `u16`
///
/// Useful for Serde's `serialize_with` attribute
pub fn dns_response_code_serialize<S>(response_code: &DnsResponseCode, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
  serializer.serialize_u16(response_code.clone().into())
}

/// Deserializes a `u16` into a `DnsResponseCode`
///
/// Useful for Serde's `deserialize_with` attribute
pub fn dns_response_code_deserialize<'de, D>(deserializer: D) -> Result<DnsResponseCode, D::Error> where D: Deserializer<'de> {
  let raw_response_code: u16 = Deserialize::deserialize(deserializer)?;
  Ok(raw_response_code.into())
}

/// Serializes a `DnsRecordType` into a `u16`
///
/// Useful for Serde's `serialize_with` attribute
pub fn dns_record_type_serialize<S>(record_type: &DnsRecordType, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
  serializer.serialize_u16(record_type.clone().into())
}

/// Deserializes a `u16` into a `DnsRecordType`
///
/// Useful for Serde's `deserialize_with` attribute
pub fn dns_record_type_deserialize<'de, D>(deserializer: D) -> Result<DnsRecordType, D::Error> where D: Deserializer<'de> {
  let raw_record_type: u16 = Deserialize::deserialize(deserializer)?;
  Ok(raw_record_type.into())
}

#[cfg(test)]
mod test {
  use super::*;
  use std::{io::Read, fs::File, path::Path};

  fn read_file_to_vec<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    buf
  }

  #[test]
  fn should_deserialize_udp_query_example_com() {
    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_A-example.com-packet.bin");
    let dns_req_result = dns_message_from_bytes(&buf);

    assert!(dns_req_result.is_ok());
    let dns_req = dns_req_result.unwrap();

    assert_eq!(dns_req.message_type(), DnsMessageType::Query);
    assert!(dns_req.id() > 0);
    assert_eq!(dns_req.query_count(), 1);
    assert_eq!(dns_req.answer_count(), 0);
    assert_eq!(dns_req.additional_count(), 1);
    assert_eq!(dns_req.name_server_count(), 0);

    let dns_query = &(dns_req.queries())[0];
    assert_eq!(dns_query.query_type(), DnsRecordType::A);
    assert_eq!(dns_query.query_class(), DnsClass::IN);
    assert_eq!(dns_query.name().to_utf8(), "example.com.");

    assert!(dns_req.edns().is_some());
    let edns = dns_req.edns().unwrap();
    assert_eq!(edns.version(), 0);
    assert_eq!(edns.options().options().len(), 1);
    assert!(edns.options().options().contains_key(&DnsRDataOPTCode::Cookie));
  }

  #[test]
  fn should_deserialize_udp_query_noedns_example_com() {
    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_noedns_A-example.com-packet.bin");
    let dns_req_result = dns_message_from_bytes(&buf);

    assert!(dns_req_result.is_ok());
    let dns_req = dns_req_result.unwrap();

    assert_eq!(dns_req.message_type(), DnsMessageType::Query);
    assert!(dns_req.id() > 0);
    assert_eq!(dns_req.query_count(), 1);
    assert_eq!(dns_req.answer_count(), 0);
    assert_eq!(dns_req.additional_count(), 0);
    assert_eq!(dns_req.name_server_count(), 0);

    let dns_query = &(dns_req.queries())[0];
    assert_eq!(dns_query.query_type(), DnsRecordType::A);
    assert_eq!(dns_query.query_class(), DnsClass::IN);
    assert_eq!(dns_query.name().to_utf8(), "example.com.");

    assert!(dns_req.edns().is_none());
  }

  #[test]
  fn should_deserialize_udp_query_aaaa_www_ivandemarino_me() {
    let buf = read_file_to_vec("./test/fixtures/dns_udp_query_AAAA-www.ivandemarino.me-packet.bin");
    let dns_req_result = dns_message_from_bytes(&buf);

    assert!(dns_req_result.is_ok());
    let dns_req = dns_req_result.unwrap();

    assert_eq!(dns_req.message_type(), DnsMessageType::Query);
    assert!(dns_req.id() > 0);
    assert_eq!(dns_req.query_count(), 1);
    assert_eq!(dns_req.answer_count(), 0);
    assert_eq!(dns_req.additional_count(), 1);
    assert_eq!(dns_req.name_server_count(), 0);

    let dns_query = &(dns_req.queries())[0];
    assert_eq!(dns_query.query_type(), DnsRecordType::AAAA);
    assert_eq!(dns_query.query_class(), DnsClass::IN);
    assert_eq!(dns_query.name().to_utf8(), "www.ivandemarino.me.");

    assert!(dns_req.edns().is_some());
    let edns = dns_req.edns().unwrap();
    assert_eq!(edns.version(), 0);
    assert_eq!(edns.options().options().len(), 1);
    assert!(edns.options().options().contains_key(&DnsRDataOPTCode::Cookie));
  }
}