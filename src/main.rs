#[macro_use] extern crate log;
extern crate exitcode;
extern crate mooncell;

use std::{sync::mpsc::{Sender, Receiver, channel}, process};

use mooncell::{logging, net::{server::Server, request::Request}, config::{cli::CLI, config::Config}, core::processor::Processor};

fn main() {
  // Load CLI configuration and initialize logging
  let cli = CLI::new();
  logging::init(&cli);
  trace!("{:#?}", cli);

  if cli.is_list_providers() {
    // Sub-command "list providers" was invoked
    cli.list_providers();
  } else if cli.provider().is_none() {
    // No provider was configured: wrong usage
    process::exit(exitcode::USAGE);
  } else {
    // OK, Server can run!
    info!("DNS Server starting");

    let (sender, receiver): (Sender<Request>, Receiver<Request>) = channel();
    let processor = Processor::new(receiver, cli.resolver());

    let mut server = Server::new(&cli, sender);
    server.start();

//    // TODO Temporary: just used to prove cross-thread comms
//    for req in receiver.iter() {
//      debug!("{:#?}", req);
//    }

    // TODO Build the right DoHResolver, based on the given Config
    // TODO Create Processor by passing in:
    //   - Receiver<Request>
    //   - DoHResolver

    server.await_termination_and_drop();
    info!("Shutting down");
  }
}
