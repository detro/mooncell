#[macro_use] extern crate log;
extern crate mooncell;

use std::sync::mpsc::{Sender, Receiver, channel};

use mooncell::{logging, net::{server::DnsServer, request::DnsRequest}, config::{cli::CLI}};

fn main() {
  let cli = CLI::new();
  println!("{:#?}", cli);

  logging::init(&cli);
  info!("DNS Server starting");

  let (sender, receiver): (Sender<DnsRequest>, Receiver<DnsRequest>) = channel();

  let mut server = DnsServer::new(&cli, sender);
  server.start();

  // TODO Temporary: just used to prove cross-thread comms
  for req in receiver.iter() {
    debug!("{:#?}", req);
  }

  server.await_termination_and_drop();
  info!("Shutting down");
}
