//! Configuration Provider trait (schema)

use log::LevelFilter;
use std::net::{Ipv4Addr, Ipv6Addr};

/// This trait is implemented by types that _provide configuration_ to the rest of the application.
pub trait ConfigProvider {
  /// The IPv4 to bind to
  fn ipv4(&self) -> Vec<Ipv4Addr>;

  /// The IPv6 to bind to
  fn ipv6(&self) -> Vec<Ipv6Addr>;

  /// The port to listen on
  fn port(&self) -> u16;

  /// The log level filter to use
  fn log_filter(&self) -> LevelFilter;
}