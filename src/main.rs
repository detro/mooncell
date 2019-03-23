#[macro_use] extern crate log;
extern crate exitcode;
extern crate crossbeam_channel;
extern crate mooncell;

use mooncell::{logging, net::{server::Server, request::Request}, config::{cli::CLI, config::Config}, core::processor::Processor};
use crossbeam_channel::{Sender as XBeamSender, Receiver as XBeamReceiver, self as xbeam_channel};
use std::process;

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
    info!("Starting...");

    // Create the channel for Server -> Processor communication

    let (sender, receiver): (XBeamSender<Request>, XBeamReceiver<Request>) = xbeam_channel::unbounded();
    // Create Processor (i.e. "consumer")
    let mut processor = Processor::new(receiver, cli.resolver());
    // Create Server (i.e. "producer")
    let mut server = Server::new(&cli, sender);

    // TODO Register signal handler to:
    //  1. stop server
    //  2. stop processor
    //  3. info!("... shut down");

    // Launch key services
    processor.start();
    server.start();

    // Graceful termination
    server.await_termination();
    processor.await_termination();
  }
}
