use std::net::{SocketAddr, TcpStream, UdpSocket};
use dns::protocol::{DnsMessage, dns_message_to_bytes};

/// The type of `DnsRequest`
#[derive(Debug)]
enum DnsRequestType {
  UdpRequest,
  TcpRequest
}

/// A Request received by the `DnsServer`
///
/// The `DnsServer` handles both UDP and TCP requests, so this struct has a `req_type` field
/// to distinguish of which nature the request is. The _type_ also determines if the fields
/// `tcp_stream` or `udp_socket` are populated: they are mutually exclusive.
#[derive(Debug)]
pub struct DnsRequest {
  source: SocketAddr,
  dns_query: DnsMessage,
  req_type: DnsRequestType,
  tcp_stream: Option<TcpStream>,
  udp_socket: Option<UdpSocket>,
}

impl DnsRequest {

  /// Constructor for a new `DnsRequestResponder` of type `UdpRequest`
  ///
  /// This is to be used when a UDP request is received.
  ///
  /// # Parameters
  ///
  /// * `source` - Socket address source
  /// * `dns_query` - DNS Message received from the given source
  /// * `socket` - UDP Socket from which the Message was received and a response can be sent
  pub fn from_udp(source: SocketAddr, dns_query: DnsMessage, socket: UdpSocket) -> DnsRequest {
    DnsRequest {
      source,
      dns_query,
      req_type: DnsRequestType::UdpRequest,
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
  pub fn from_tcp(source: SocketAddr, dns_query: DnsMessage, stream: TcpStream) -> DnsRequest {
    DnsRequest {
      source,
      dns_query,
      req_type: DnsRequestType::TcpRequest,
      tcp_stream: Some(stream),
      udp_socket: None
    }
  }

  pub fn source(&self) -> &SocketAddr {
    &self.source
  }

  pub fn dns_query(&self) -> &DnsMessage {
    &self.dns_query
  }

  pub fn respond(self, dns_res: DnsMessage) {
    match dns_message_to_bytes(&dns_res) {
      Ok(raw_dns_res) => {
        match self.req_type {
          // Send response over UDP
          DnsRequestType::UdpRequest => {
            match self.udp_socket.unwrap().send_to(raw_dns_res.as_ref(), self.source) {
              Err(err) => {
                error!("Unable to send response back over UDP socket: {}", err);
              },
              Ok(amount) => {
                trace!("Sent back response over UDP socket, made of {} bytes", amount);
              },
            };
          },

          // Send response over TCP
          DnsRequestType::TcpRequest => {
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
