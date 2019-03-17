//! Processor...

use net::request::Request;
use core::resolver::DoHResolver;
use threadpool::{Builder as ThreadPoolBuilder, ThreadPool};
use num_cpus;
use std::sync::{atomic::{AtomicBool, Ordering}, mpsc::Receiver};

pub struct Processor {
  receiver: Receiver<Request>,
  resolver: Box<dyn DoHResolver>,
  pool: ThreadPool,
  running: AtomicBool
}

impl Processor {
  pub fn new(receiver: Receiver<Request>, resolver: Box<dyn DoHResolver>) -> Processor {
    let pool = ThreadPoolBuilder::new()
      .num_threads(num_cpus::get() * 4)
      .thread_name("Processor Thread Pool".into())
      .build();

    Processor {
      receiver,
      resolver,
      pool,
      running: AtomicBool::new(false)
    }
  }

  pub fn start(&self) {

  }

  pub fn stop(&self) {
    self.running.swap(false, Ordering::Relaxed);
    self.pool.join();
  }
}