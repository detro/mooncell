//! Processor...

use net::request::Request;
use core::resolver::DoHResolver;
use threadpool::{Builder as ThreadPoolBuilder, ThreadPool};
use num_cpus;
use crossbeam_channel::{Receiver as XBeamReceiver, RecvTimeoutError as XBeamRecvTimeoutError};
use std::{thread, time::Duration, sync::{Arc, atomic::{AtomicBool, Ordering}}};

const PROCESSOR_RECEIVER_THREAD_NAME: &'static str = "processor_receiver_thread";
const PROCESSOR_RESOLVER_THREADPOOL_NAME: &'static str = "processor_resolver_threadpool";
const PROCESSOR_THREADS_COUNT_CPU_MULTIPLIER: usize = 4;
const PROCESSOR_RECEIVER_TIMEOUT_SEC: u64 = 10;

pub struct Processor {
  receiver: XBeamReceiver<Request>,
  resolver: Box<DoHResolver + Send>,
  is_running: Arc<AtomicBool>,
}

impl Processor {
  pub fn new(receiver: XBeamReceiver<Request>, resolver: Box<DoHResolver + Send>) -> Processor {
    Processor {
      receiver,
      resolver,
      is_running: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn start(&mut self) {
    // Start the Processor
    self.do_start_running();

    let receiver = self.receiver.clone();
    let resolver = self.resolver.clone();
    let is_running = self.is_running.clone();

    // Launch a 'request receiving' thread
    thread::Builder::new().name(PROCESSOR_RECEIVER_THREAD_NAME.into()).spawn(move || {
      // Create a new thread pool
      let pool = init_pool();
      let receive_timeout_duration = Duration::from_secs(PROCESSOR_RECEIVER_TIMEOUT_SEC);

      // Receive 'requests' for processing, but interrupt at regular intervals to check if Processor was stopped
      loop {
        match receiver.recv_timeout(receive_timeout_duration) {
          Ok(req) => {
            let resolver = resolver.clone();
            pool.execute(move || resolve_and_respond(req, resolver));
          },
          Err(XBeamRecvTimeoutError::Timeout) => {
            if !is_running.load(Ordering::Relaxed) {
              trace!("Processor is done running: stop receiving requests");
              break;
            }
          },
          Err(err) => {
            error!("Unexpected error when receiving requests: {}", err);
          }
        }
      }

      debug!("Wait for any pending processing...");
      pool.join();
      debug!("... done processing");
    }).expect(format!("Unable to spawn {}", PROCESSOR_RECEIVER_THREAD_NAME).as_ref());
  }

  pub fn stop(&mut self) {
    trace!("Processor should now stop...");
    self.do_stop_running();
  }

  fn do_start_running(&mut self) {
    self.is_running.swap(true, Ordering::Relaxed);
  }

  fn do_stop_running(&mut self) {
    self.is_running.swap(false, Ordering::Relaxed);
  }
}

fn init_pool() -> ThreadPool {
  ThreadPoolBuilder::new()
    .num_threads(num_cpus::get() * PROCESSOR_THREADS_COUNT_CPU_MULTIPLIER)
    .thread_name(PROCESSOR_RESOLVER_THREADPOOL_NAME.into())
    .build()
}

fn resolve_and_respond(req: Request, resolver: Box<DoHResolver>) -> () {
  match resolver.resolve(req.dns_query()) {
    Ok(res_msg) => {
      req.respond(res_msg);
    },
    Err(err) => {
      error!("Unable to resolve request: {}", err);
    }
  }
}