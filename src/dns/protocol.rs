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
  },
  rr::{
    record_type::RecordType as DnsRecordType,
    dnssec::rdata::DNSSECRecordType as DnsDNSSECRecordType,
    dns_class::DNSClass as DnsClass,
  }
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