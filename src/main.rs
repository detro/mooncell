#[macro_use] extern crate log;
extern crate mooncell;

use std::sync::mpsc::{Sender, Receiver, channel};

use mooncell::{logging, net::{server::Server, request::Request}, config::{cli::CLI}};

fn main() {
  let cli = CLI::new();
  println!("{:#?}", cli);

  logging::init(&cli);
  info!("DNS Server starting");

  let (sender, receiver): (Sender<Request>, Receiver<Request>) = channel();

  let mut server = Server::new(&cli, sender);
  server.start();

  // TODO Temporary: just used to prove cross-thread comms
  for req in receiver.iter() {
    debug!("{:#?}", req);
  }

  server.await_termination_and_drop();
  info!("Shutting down");
}
