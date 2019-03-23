//! Processing of received requests

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
  is_running: Arc<AtomicBool>,
  receiver_thread_handle: Option<thread::JoinHandle<()>>,
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
      is_running: Arc::new(AtomicBool::new(false)),
      receiver_thread_handle: None,
    }
  }

  /// Start the Processor
  ///
  /// NOTE: This method is asynchronous (i.e. non blocking)
  pub fn start(&mut self) {
    // Start the Processor
    self.do_start_running();

    let receiver = self.receiver.clone();
    let resolver = self.resolver.clone();
    let is_running = self.is_running.clone();

    // Launch a 'request receiving' thread
    self.receiver_thread_handle = Some(thread::Builder::new()
      .name(PROCESSOR_RECEIVER_THREAD_NAME.into())
      .spawn(move || {
        // Create a new thread pool
        let pool = init_pool();
        let receive_timeout_duration = Duration::from_secs(PROCESSOR_RECEIVER_TIMEOUT_SEC);

        // Receive 'requests' for processing, but interrupt at regular intervals to check if Processor was stopped
        loop {
          match receiver.recv_timeout(receive_timeout_duration) {
            Ok(req) => {
              {
                let q = req.dns_query();
                debug!("Received: id={} type={:?} queries={:?}", q.id(), q.message_type(), q.queries());
              }

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
      })
      .expect(format!("Unable to spawn thread: {}", PROCESSOR_RECEIVER_THREAD_NAME).as_ref())
    );
  }

  /// Stop the Processor
  ///
  /// NOTE: This method is asynchronous (i.e. non blocking)
  pub fn stop(&mut self) {
    trace!("Processor should now stop...");
    self.do_stop_running();
  }

  /// Away Processor termination
  ///
  /// NOTE: This consumes the Processor instance
  pub fn await_termination(self) {
    // Wait for receiver thread to stop, only if it's actually set
    if self.receiver_thread_handle.is_some() {
      self.receiver_thread_handle.unwrap()
        .join()
        .expect(format!("Panicked upon termination: {}", PROCESSOR_RECEIVER_THREAD_NAME).as_ref());
    }
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
      debug!("Responding: id={} type={:?} answers={:?}", res_msg.id(), res_msg.message_type(), res_msg.answers());
      req.respond(res_msg);
    },
    Err(err) => {
      error!("Unable to resolve request: {}", err);
    }
  }
}