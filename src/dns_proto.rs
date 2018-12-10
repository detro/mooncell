//! Mode that re-publishes the part of Trust-DNS Proto that we care about

pub use trust_dns_proto::error::{ProtoError, ProtoErrorKind};
pub use trust_dns_proto::op::message::Message;

use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};

pub fn message_from_bytes(bytes: &[u8]) -> Result<Message, ProtoError> {
  Message::from_bytes(bytes)
}

pub fn message_to_bytes(message: &Message) -> Result<Vec<u8>, ProtoError> {
  message.to_bytes()
}