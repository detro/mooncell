//! Implementation of a Server, that listen for both UDP and TCP DNS queries
//!
//! It's role is to handle the networking part of receiving a DNS queries

use crate::config::config::Config;
use super::{utils::{bind_udp_sockets, /*bind_tcp_listeners*/}, request::Request};
use crate::dns;

use log::*;
use crossbeam_channel::Sender as XBeamSender;
use srvzio;

use std::{net::{Ipv4Addr, Ipv6Addr, UdpSocket}, thread, time::Duration, io::ErrorKind};

const SERVER_SERVICE_NAME: &'static str = "Server";

/// The DNS Server that listens for DNS queries over UDP or TCP requests.
#[derive(Debug)]
pub struct Server {
  ip4s: Vec<Ipv4Addr>,
  ip6s: Vec<Ipv6Addr>,
  port: u16,
  threads: Vec<thread::JoinHandle<()>>,
  sender: XBeamSender<Request>,
  status: srvzio::ServiceStatusFlag,
}

impl srvzio::Service for Server {

  fn name(&self) -> &'static str {
    SERVER_SERVICE_NAME
  }


  fn start(&mut self) {
    self.status.starting();

    // Bind TCP listeners and start dedicated threads to handle requests (one thread per listener)
    // TODO Implement TCP support
//    let tcp_listeners = bind_tcp_listeners(&self.ip4s, &self.ip6s, &self.port);
//    let threads = self.start_tcp_threads(tcp_listeners);

    // Bind UDP sockets and start dedicated threads to listen for requests (one thread per socket)
    let udp_sockets = bind_udp_sockets(&self.ip4s, &self.ip6s, &self.port);
    let threads = self.start_udp_threads(udp_sockets);

    self.threads.extend(threads);
  }

  fn await_started(&mut self) {
    while !self.status.is_started() {}
  }

  fn stop(&mut self) {
    trace!("Server should now stop...");
    self.status.stopping();
  }

  fn await_stopped(&mut self) {
    while let Some(t) = self.threads.pop() {
      t.join()
        .expect("A Server's thread panicked upon termination");
    }

    self.status.stopped();
  }
}

impl Server {

  /// Constructor
  ///
  /// # Parameters
  ///
  /// * `config` - Configuration to be used by the `DnsServer` when started
  /// * `sender` - Channel sender to "emit" `DnsRequest` after been received and parsed by the Server
  pub fn new(config: &Config, sender: XBeamSender<Request>) -> Server {
    Server {
      ip4s: config.ipv4(),
      ip6s: config.ipv6(),
      port: config.port(),
      threads: Vec::with_capacity(config.ipv4().len() + config.ipv6().len()),
      sender,
      status: srvzio::ServiceStatusFlag::default(),
    }
  }

  /// Spawn threads dedicated to handle `UdpSocket` traffic
  ///
  /// This method will allocate 1 `Thread` per `UdpSocket` given as input.
  /// For the focused reader this sounds like
  /// _"we are accepting just 1 UDP request at a time per bound socket"_, but this is how
  /// UDP works.
  ///
  /// That's why the purpose of these threads is to receive the request, deserialize it
  /// into a `DnsMessage` and then emit on the given internal `self.sender` channel for
  /// further processing.
  ///
  /// The consumption of those emitted entities can then be parallelized as desired/needed,
  /// to speed things up.
  fn start_udp_threads(&mut self, udp_sockets: Vec<UdpSocket>) -> Vec<thread::JoinHandle<()>> {
    // Map the bound sockets to threads, so we can later on use their `JoinHandle` to terminate them
    udp_sockets.iter().enumerate().map(|(idx, udp_sock)| {

      let thread_udp_sock = udp_sock.try_clone().unwrap();
      let thread_udp_sender = self.sender.clone();
      let status = self.status.clone();

      // Launch a thread per socket we are listening on
      thread::Builder::new().name(format!("udp_socket_thread_{}", idx)).spawn(move || {
        let mut buf: [u8; 512] = [0; 512];

        // Set read timeout on the UDP socket (so we can actually stop this thread)
        thread_udp_sock
          .set_read_timeout(Some(Duration::from_secs(10))) //< TODO Make this configurable
          .expect("Unable to set read timeout on UDP socket");

        status.started();

        trace!("Waiting for UDP datagram...");
        loop {
          match thread_udp_sock.recv_from(&mut buf) {
            Ok((amount, src)) => {
              debug!("Received {} bytes via UDP datagram from '{}'", amount, src);

              match dns::protocol::dns_message_from_bytes(&buf) {
                Ok(dns_message) => {
                  if dns_message.message_type() == dns::protocol::DnsMessageType::Query {
                    let u_sock = thread_udp_sock.try_clone().unwrap();
                    let dns_request = Request::from_udp(src, dns_message, u_sock);

                    thread_udp_sender.send(dns_request).expect("Unable to pass on DNS Request for processing");
                  } else {
                    warn!("Received unexpected DNS message of type {:?}: ignoring", dns_message.message_type());
                  }
                },
                Err(e) => error!("Unable to parse DNS message: {}", e)
              };
            }
            Err(e) => match e.kind() {
              ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                // NOTE: This happens when `recv_from` has timed out: unless `DnsServer` has been
                // stopped, we need to resume listening for requests.
                // Why 2 errors? Because on Unix systems the error would be `WouldBlock`, while
                // on Windows systems the error would be `TimedOut`

                if status.is_stopping() {
                  trace!("Server is done running: stop listening for requests");
                  break;
                }
              },
              _ => {
                error!("Error receiving: {:?} {}", e.kind(), e);
              }
            }
          }
        }
      }).expect("Unable to spawn thread for UDP bound socket")
    }).collect()
  }

}
