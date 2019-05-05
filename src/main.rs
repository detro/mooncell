mod config;
mod core;
mod dns;
mod doh_json;
mod doh_wire;
mod net;
mod logging;

use crate::net::{server::Server, request::Request};
use crate::config::{cli::CLI, config::Config};
use crate::core::processor::Processor;

use log::*;
use crossbeam_channel::{Sender as XBeamSender, Receiver as XBeamReceiver, self as xbeam_channel};
use srvzio::Service;
use exitcode;

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
    let mut srv_mgr = srvzio::ServiceManager::new();

    // Create Processor: the "consumer" of requests
    srv_mgr.register(Box::new(Processor::new(receiver, cli.resolver())));
    // Create Server: the "producer" of requests
    srv_mgr.register(Box::new(Server::new(&cli, sender)));

    srv_mgr.start_and_await();

    srv_mgr.await_termination_signal_then_stop();

    info!("... Terminated.");
  }
}
