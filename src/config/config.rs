//! Configuration Provider trait (schema)

use log::LevelFilter;
use std::net::{Ipv4Addr, Ipv6Addr};
use core::protocol::DoHProtocol;

/// This trait is implemented by types that _provide configuration_ to the rest of the application.
pub trait Config {
  /// The IPv4 to bind to
  fn ipv4(&self) -> Vec<Ipv4Addr>;

  /// The IPv6 to bind to
  fn ipv6(&self) -> Vec<Ipv6Addr>;

  /// The port to listen on
  fn port(&self) -> u16;

  /// The log level filter to use
  fn log_filter(&self) -> LevelFilter;

  /// The DNS-over-HTTPS Protocol to use
  fn protocol(&self) -> DoHProtocol;

  /// The identifier of DNS-over-HTTPS Provider to use
  fn provider(&self) -> &'static str;
}
