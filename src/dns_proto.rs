//! Module that re-publishes the part of Trust-DNS Proto that we care about, and adds some utilities
//! to avoid having to use `trust_dns_proto` in other modules.

pub use trust_dns_proto::error::{ProtoError, ProtoErrorKind};
pub use trust_dns_proto::op::message::Message;

use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};

/// Converts an array of raw bytes into a DNS `Message`
pub fn message_from_bytes(bytes: &[u8]) -> Result<Message, ProtoError> {
  Message::from_bytes(bytes)
}

/// Converts a DNS `Message` to a vector of raw bytes
pub fn message_to_bytes(message: &Message) -> Result<Vec<u8>, ProtoError> {
  message.to_bytes()
}