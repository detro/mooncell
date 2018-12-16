//! Module that re-publishes the part of Trust-DNS Proto that we care about, and adds some utilities
//! to avoid having to use `trust_dns_proto` in other modules.

pub use trust_dns_proto::error::{ProtoError as DnsProtoError, ProtoErrorKind as DnsProtoErrorKind};
pub use trust_dns_proto::op::message::Message as DnsMessage;

use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};

/// Converts an array of raw bytes into a DNS `Message`
pub fn message_from_bytes(bytes: &[u8]) -> Result<DnsMessage, DnsProtoError> {
  DnsMessage::from_bytes(bytes)
}

/// Converts a DNS `Message` to a vector of raw bytes
pub fn message_to_bytes(message: &DnsMessage) -> Result<Vec<u8>, DnsProtoError> {
  message.to_bytes()
}