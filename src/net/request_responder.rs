use std::net::{SocketAddr, TcpStream, UdpSocket};
use dns::protocol::DnsMessage;
use doh_json::protocol::DoHResponse;

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
pub struct DnsRequestResponder {
  source: SocketAddr,
  message: DnsMessage,
  req_type: DnsRequestType,
  tcp_stream: Option<TcpStream>,
  udp_socket: Option<UdpSocket>,
}

impl DnsRequestResponder {

  /// Constructor for a new `DnsRequestResponder` of type `UdpRequest`
  ///
  /// This is to be used when a UDP request is received.
  ///
  /// # Parameters
  ///
  /// * `src` - Socket address source
  /// * `msg` - DNS Message received from the given source
  /// * `sock` - UDP Socket from which the Message was received and a response can be sent
  pub fn from_udp_request(src: SocketAddr, msg: DnsMessage, sock: UdpSocket) -> DnsRequestResponder {
    DnsRequestResponder {
      source: src,
      message: msg,
      req_type: DnsRequestType::UdpRequest,
      tcp_stream: None,
      udp_socket: Some(sock)
    }
  }

  /// Constructor for a new `DnsRequestResponder` of type `TcpRequest`
  ///
  /// This is to be used when a TCP request is received.
  ///
  /// # Parameters
  ///
  /// * `src` - Socket address source
  /// * `msg` - DNS DnsMessage received from the given source
  /// * `strm` - TCP Stream representing a live and established connection with the source
  pub fn from_tcp_request(src: SocketAddr, msg: DnsMessage, strm: TcpStream) -> DnsRequestResponder {
    DnsRequestResponder {
      source: src,
      message: msg,
      req_type: DnsRequestType::TcpRequest,
      tcp_stream: Some(strm),
      udp_socket: None
    }
  }

  pub fn source(&self) -> &SocketAddr {
    &self.source
  }

  pub fn message(&self) -> &DnsMessage {
    &self.message
  }

  pub fn respond(&self, response: DoHResponse) {
    // TODO

    match &self.req_type {
      DnsRequestType::UdpRequest => {
        // TODO
        unimplemented!();
      },
      DnsRequestType::TcpRequest => {
        // TODO
        unimplemented!();
      }
    };
  }

}

//fn response_to_response(doh_response: DoHResponse) -> DnsMessage {
//  let dns_response = DnsMessage::new();
//
//  doh_response.
//
//  dns_response
//}

#[cfg(test)]
mod test {
  use super::*;

  // TODO
}