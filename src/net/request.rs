//! Implementation of a Request, with minimal logic to differentiate UDP and TCP DNS queries and responses
//!
//! It's role is to wrap the received DNS query and provide a network-abstract way to respond back

use dns::protocol::{DnsMessage, DnsMessageType, dns_message_to_bytes};
use std::net::{SocketAddr, TcpStream, UdpSocket};

/// The type of `DnsRequest`
#[derive(Debug)]
enum RequestType {
  UdpRequest,
  TcpRequest
}

/// A Request received by the `DnsServer`
///
/// The `DnsServer` handles both UDP and TCP requests, so this struct has a `req_type` field
/// to distinguish of which nature the request is. The _type_ also determines if the fields
/// `tcp_stream` or `udp_socket` are populated: they are mutually exclusive.
#[derive(Debug)]
pub struct Request {
  source: SocketAddr,
  dns_query: DnsMessage,
  req_type: RequestType,
  tcp_stream: Option<TcpStream>,
  udp_socket: Option<UdpSocket>,
}

impl Request {

  /// Constructor for a new `DnsRequestResponder` of type `UdpRequest`
  ///
  /// This is to be used when a UDP request is received.
  ///
  /// # Parameters
  ///
  /// * `source` - Socket address source
  /// * `dns_query` - DNS Message received from the given source
  /// * `socket` - UDP Socket from which the Message was received and a response can be sent
  pub fn from_udp(source: SocketAddr, dns_query: DnsMessage, socket: UdpSocket) -> Request {
    Request {
      source,
      dns_query,
      req_type: RequestType::UdpRequest,
      tcp_stream: None,
      udp_socket: Some(socket)
    }
  }

  /// Constructor for a new `DnsRequestResponder` of type `TcpRequest`
  ///
  /// This is to be used when a TCP request is received.
  ///
  /// # Parameters
  ///
  /// * `source` - Socket address source
  /// * `dns_query` - DNS DnsMessage received from the given source
  /// * `stream` - TCP Stream representing a live and established connection with the source
  pub fn from_tcp(source: SocketAddr, dns_query: DnsMessage, stream: TcpStream) -> Request {
    Request {
      source,
      dns_query,
      req_type: RequestType::TcpRequest,
      tcp_stream: Some(stream),
      udp_socket: None
    }
  }

  /// Return the "source" (i.e. `SocketAddress`) that sent the request
  pub fn source(&self) -> &SocketAddr {
    &self.source
  }

  /// Return the DNS query (i.e. `DnsMessage` of type `Query`) that was received
  pub fn dns_query(&self) -> &DnsMessage {
    &self.dns_query
  }

  /// Respond to the request with the given DNS response
  ///
  /// NOTE: Assumes the DNS Message Type is a Response, otherwise it will panic!
  ///
  /// # Parameters
  /// * `dns_res`: `DnsMessage` of type `DnsMessageType::Response`, that this call will send to the original requestor
  pub fn respond(self, dns_res: DnsMessage) {
    // Before resolving, check the type is right
    if dns_res.message_type() == DnsMessageType::Query {
      error!("A DNS Query was provided instead of a DNS Response: this is clearly a bug that needs fixing!");
      // TODO Send a empty/error DNS response (or something sensible)
    }

    match dns_message_to_bytes(&dns_res) {
      Ok(raw_dns_res) => {
        match self.req_type {
          // Send response over UDP
          RequestType::UdpRequest => {
            if let Err(err) = self.udp_socket.unwrap().send_to(raw_dns_res.as_ref(), self.source) {
              error!("Unable to send response back over UDP socket: {}", err);
              // TODO Send a empty/error DNS response (or something sensible)
            };
          },

          // Send response over TCP
          RequestType::TcpRequest => {
            // TODO Send message back on the TCP stream
            unimplemented!();
          }
        };
      },
      Err(err) => {
        error!("Unable to serialize response: {}", err);
        // TODO Send a empty/error DNS response (or something sensible)
      },
    };
  }

}
