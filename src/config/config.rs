//! Configuration Provider trait (schema)

use log::LevelFilter;
use std::net::{Ipv4Addr, Ipv6Addr};
use core::{protocol::DoHProtocol, provider::DoHProvider, resolver::DoHResolver};
use doh_json::{resolver::DoHJsonResolver, provider::DoHJsonProvider};

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

  /// The DNS-over-HTTPS Provider to use
  fn provider(&self) -> Option<Box<dyn DoHProvider>>;

  /// The DNS-over-HTTPS Resolver to use
  fn resolver(&self) -> Box<DoHResolver + Send> {
    match self.protocol() {
      DoHProtocol::JSON => match self.provider() {
        Some(provider) => Box::new(DoHJsonResolver::new(*provider.downcast::<DoHJsonProvider>().unwrap())),
        None => panic!("Unable to determine DoH JSON Provider: this should never be reached!"),
      },
      // TODO Rewrite the following once support for WIRE protocol is implemented
      DoHProtocol::WIRE => panic!("Unable to determine DoH WIRE Provider: this should never be reached!"),
    }
  }
}
