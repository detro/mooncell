//! Processing of received requests

use crate::net::request::Request;
use super::resolver::DoHResolver;

use log::*;
use threadpool::{Builder as ThreadPoolBuilder, ThreadPool};
use num_cpus;
use crossbeam_channel::{Receiver as XBeamReceiver, RecvTimeoutError as XBeamRecvTimeoutError};
use srvzio;

use std::{thread, time::Duration};

const PROCESSOR_SERVICE_NAME: &'static str = "Processor";
const PROCESSOR_RECEIVER_THREAD_NAME: &'static str = "processor_receiver_thread";
const PROCESSOR_RESOLVER_THREAD_NAME: &'static str = "processor_resolver_thread";
const PROCESSOR_THREADS_COUNT_CPU_MULTIPLIER: usize = 4;
const PROCESSOR_RECEIVER_TIMEOUT_SEC: u64 = 10;

/// Processor is a service that receives and responds to DNS requests
///
/// It has internal threading to handle being "started" and "stopped": stopped at construction.
/// The `Request` are received via the `Receiver` (see `crossbeam::channel::Receiver`), and then
/// resolved via the `DoHResolver`.
///
/// The service is agnostic to what kind of DNS-over-HTTPS resolution is configured: it just
/// uses the provided `DoHResolver`.
pub struct Processor {
  receiver: XBeamReceiver<Request>,
  resolver: Box<DoHResolver + Send>,
  status: srvzio::ServiceStatusFlag,
  receiver_thread: Option<thread::JoinHandle<()>>,
}

impl Processor {

  /// Constructor
  ///
  /// # Parameters
  /// * `receiver`: a `crossbeam::channel::Receiver` that delivers `Request` data
  /// * `resolver`: a struct that implements the `DoHResolver`, wrapped in a `Box`
  pub fn new(receiver: XBeamReceiver<Request>, resolver: Box<DoHResolver + Send>) -> Processor {
    Processor {
      receiver,
      resolver,
      status: srvzio::ServiceStatusFlag::default(),
      receiver_thread: None,
    }
  }

}

impl srvzio::Service for Processor {

  fn name(&self) -> &'static str {
    PROCESSOR_SERVICE_NAME
  }


  fn start(&mut self) {
    self.status.starting();

    let receiver = self.receiver.clone();
    let resolver = self.resolver.clone();
    let status = self.status.clone();

    // Launch a 'request receiving' thread
    self.receiver_thread = Some(thread::Builder::new()
      .name(PROCESSOR_RECEIVER_THREAD_NAME.into())
      .spawn(move || {
        let pool = crate_thread_pool();

        status.started();

        // Receive 'requests' for processing, but interrupt at regular intervals to check if Processor was stopped
        loop {
          match receiver.recv_timeout(Duration::from_secs(PROCESSOR_RECEIVER_TIMEOUT_SEC)) {
            Ok(req) => {
              {
                let q = req.dns_query();
                let s = req.source();
                debug!("Received: id={} type={:?} source={} queries={:?}", q.id(), q.message_type(), s, q.queries());
              }

              let resolver = resolver.clone();
              pool.execute(move || resolve_and_respond(req, resolver));
            },
            Err(XBeamRecvTimeoutError::Timeout) => {
              if status.is_stopping() {
                trace!("{} is done running: stop processing requests", PROCESSOR_SERVICE_NAME);
                break;
              }
            },
            Err(err) => {
              error!("Unexpected error when processing requests: {}", err);
            }
          }
        }

        debug!("Wait for any pending processing...");
        pool.join();
        debug!("... done processing");

        status.stopped();
      })
      .expect(format!("Unable to spawn thread: {}", PROCESSOR_RECEIVER_THREAD_NAME).as_ref())
    );
  }

  fn await_started(&mut self) {
    while !self.status.is_started() {}
  }

  fn stop(&mut self) {
    trace!("{} should now stop...", PROCESSOR_SERVICE_NAME);
    self.status.stopping();
  }

  fn await_stopped(&mut self) {
    while !self.status.is_stopped() {}

    // Wait for receiver thread to stop (if it's actually set)
    if self.receiver_thread.is_some() {
      self.receiver_thread
        .take()
        .unwrap()
        .join()
        .expect(format!("Panicked upon termination: {}", PROCESSOR_RECEIVER_THREAD_NAME).as_ref());
    }
  }

}

fn crate_thread_pool() -> ThreadPool {
  ThreadPoolBuilder::new()
    .num_threads(num_cpus::get() * PROCESSOR_THREADS_COUNT_CPU_MULTIPLIER)
    .thread_name(PROCESSOR_RESOLVER_THREAD_NAME.into())
    .build()
}

fn resolve_and_respond(req: Request, resolver: Box<DoHResolver>) -> () {
  match resolver.resolve(req.dns_query()) {
    Ok(res_msg) => {
      debug!("Responding: id={} type={:?} answers={:?}", res_msg.id(), res_msg.message_type(), res_msg.answers());
      req.respond(res_msg);
    },
    Err(err) => {
      error!("Unable to resolve request: {}", err);
    }
  }
}