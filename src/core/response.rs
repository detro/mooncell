//! Trait definition for DNS-over-HTTPS response

use dns::protocol::*;
use std::{str::FromStr, string::ToString};

/// Trait defining a _response_ to a DNS Message query
pub trait DoHResponse: FromStr + ToString + Default {

  /// Apply the response to the given `DnsMessage`
  ///
  /// # Parameters
  ///
  /// * `req_edns_client_subnet_prefix_len`: length of EDNS Client Subnet used in the request
  /// * `res_dns_msg`: Response DNS Message
  fn apply(&self, req_edns_client_subnet_prefix_len: u8, res_dns_msg: &mut DnsMessage);

}