use std::net::{SocketAddr, TcpStream, UdpSocket};
use dns_proto::Message;

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
  message: Message,
  req_type: DnsRequestType,
  tcp_stream: Option<TcpStream>,
  udp_socket: Option<UdpSocket>,
}

impl DnsRequest {

  /// Constructor for a new `DnsRequest` of type `UdpRequest`
  ///
  /// This is to be used when a UDP request is received.
  ///
  /// # Parameters
  ///
  /// * `src` - Socket address source
  /// * `msg` - DNS Message received from the given source
  /// * `sock` - UDP Socket from which the message was received and a response can be sent
  pub fn new_udp_request(src: SocketAddr, msg: Message, sock: UdpSocket) -> DnsRequest {
    DnsRequest {
      source: src,
      message: msg,
      req_type: DnsRequestType::UdpRequest,
      tcp_stream: None,
      udp_socket: Some(sock)
    }
  }

  /// Constructor for a new `DnsRequest` of type `TcpRequest`
  ///
  /// This is to be used when a TCP request is received.
  ///
  /// # Parameters
  ///
  /// * `src` - Socket address source
  /// * `msg` - DNS Message received from the given source
  /// * `stream` - TCP Stream representing a live and established connection with the source
  pub fn new_tcp_request(src: SocketAddr, msg: Message, stream: TcpStream) -> DnsRequest {
    DnsRequest {
      source: src,
      message: msg,
      req_type: DnsRequestType::TcpRequest,
      tcp_stream: Some(stream),
      udp_socket: None
    }
  }

}