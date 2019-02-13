#[macro_use] extern crate log;
extern crate exitcode;
extern crate mooncell;

use std::{sync::mpsc::{Sender, Receiver, channel}, process};

use mooncell::{logging, net::{server::Server, request::Request}, config::{cli::CLI, config::Config}};

fn main() {
  // Load CLI configuration and initialize logging
  let cli = CLI::new();
  logging::init(&cli);
  trace!("{:#?}", cli);

  // Figure out if a sub-command was invoked, or we can actually start the server
  if cli.is_list_providers() {
    cli.list_providers();
  } else if cli.provider().is_none() {
    process::exit(exitcode::USAGE);
  } else {
    info!("DNS Server starting");

    let (sender, receiver): (Sender<Request>, Receiver<Request>) = channel();

    let mut server = Server::new(&cli, sender);
    server.start();

    // TODO Temporary: just used to prove cross-thread comms
    for req in receiver.iter() {
      debug!("{:#?}", req);
    }

    // TODO Build the right DoHResolver, based on the given Config
    // TODO Create Processor by passing in:
    //   - Receiver<Request>
    //   - DoHResolver

    server.await_termination_and_drop();
    info!("Shutting down");
  }
}
