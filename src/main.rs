#[macro_use] extern crate log;
extern crate mooncell;

use mooncell::{logging, net::server::DnsServer, config::{cli::CLI}};

fn main() {
  let cli = CLI::new();
  println!("{:#?}", cli);

  logging::init(&cli);
  info!("DNS Server starting");

  let mut server = DnsServer::new(&cli);
  server.start();

  server.await_termination_and_drop();
  info!("Shutting down");
}
