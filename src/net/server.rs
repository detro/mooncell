use std::{net::{Ipv4Addr, Ipv6Addr}, thread, time::Duration, io::ErrorKind, sync::mpsc::Sender};

use config::config_provider::ConfigProvider;
use net::{utils::{bind_udp_sockets, bind_tcp_listeners}, request_responder::DnsRequestResponder};
use dns_proto;

/// The DNS Server that listens for DNS queries over UDP or TCP requests.
#[derive(Debug)]
pub struct DnsServer {
  ip4s: Vec<Ipv4Addr>,
  ip6s: Vec<Ipv6Addr>,
  port: u16,
  threads: Vec<thread::JoinHandle<()>>,
  sender: Sender<DnsRequestResponder>,
}

impl DnsServer {

  /// Constructor
  ///
  /// # Parameters
  ///
  /// * `config` - Configuration to be used by the `DnsServer` when started
  /// * `sender` - Channel sender to "emit" `DnsRequest` after been received and parsed by the Server
  pub fn new<C: ConfigProvider>(config: &C, sender: Sender<DnsRequestResponder>) -> DnsServer {
    DnsServer {
      ip4s: config.ipv4(),
      ip6s: config.ipv6(),
      port: config.port(),
      threads: Vec::with_capacity(config.ipv4().len() + config.ipv6().len()),
      sender,
    }
  }

  /// Start the `DnsServer`
  ///
  /// The server binds to the IPs and ports (based on the given configuration) and
  /// it spawns a dedicated thread per binding.
  pub fn start(&mut self) {
    // TODO Add support for TCP
//    let tcp_listeners = bind_tcp_listeners(&self.ip4s, &self.ip6s, &self.port);

    // Bind UDP sockets
    let udp_sockets = bind_udp_sockets(&self.ip4s, &self.ip6s, &self.port);

    // Map the bound sockets to threads, so we can later on use their `JoinHandle` to terminate them
    let threads: Vec<thread::JoinHandle<()>> = udp_sockets.iter().enumerate().map(|(idx, udp_sock)| {
      // This creates a clone that refers to the underlying socket, but that we can use
      // to move to another thread.
      let thread_udp_sock = udp_sock.try_clone().unwrap();
      let thread_udp_sender = self.sender.clone();

      // Launch a thread per socket we are listening on
      thread::Builder::new().name(format!("udp_socket_thread_{}", idx)).spawn(move || {
        let mut buf: [u8; 512] = [0; 512];

        // Set read timeout on the UDP socket (so we can actually stop this thread)
        thread_udp_sock
          .set_read_timeout(Some(Duration::from_secs(30))) //< TODO Make this configurable
          .expect("Unable to set read timeout on UDP socket");

        trace!("Waiting for UDP datagram...");
        loop {
          match thread_udp_sock.recv_from(&mut buf) {
            Ok((amount, src)) => {
              trace!("Received {} bytes from {}", amount, src);

              match dns_proto::message_from_bytes(&buf) {
                Ok(message) => {
                  let u_sock = thread_udp_sock.try_clone().unwrap();
                  let dns_request = DnsRequestResponder::from_udp_request(src, message, u_sock);

                  thread_udp_sender.send(dns_request).expect("Unable to pass on DNS Request for processing");
                },
                Err(e) => error!("Unable to parse DNS message: {}", e)
              };
            }
            Err(e) => match e.kind() {
              ErrorKind::WouldBlock => {
                // This happens when `recv_from` has timed out: unless `DnsServer` has been
                // stopped, we need to resume listening for requests

                // TODO call `break` here once `DnsServer::stop()` has been called
              }
              _ => {
                error!("Error receiving: {}", e);
              }
            }
          }
        }
      }).expect("Unable to spawn thread for UDP bound socket")
    }).collect();

    self.threads.extend(threads);
  }

  /// Await that the DnsServer terminates and drop it
  ///
  /// The DnsServer has internal threads and so this will wait for them to be terminated,
  /// and then drop this server (because ownership is needed to call `std::thread::JoinHandle::join()`.
  pub fn await_termination_and_drop(self) {
    for t in self.threads {
      t.join().expect("Unable to join thread while awaiting termination");
    }
  }

  pub fn stop(&mut self) {
    // TODO Implement ability to 'stop' the server (by stopping the threads)
  }

}