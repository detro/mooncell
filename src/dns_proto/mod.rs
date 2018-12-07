//! Mode that re-publishes the part of Trust-DNS Proto that we care about

pub use trust_dns_proto::error::ProtoError;
pub use trust_dns_proto::op::message::Message;
pub use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};