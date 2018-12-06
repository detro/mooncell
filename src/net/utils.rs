//! Utility methods for networking

use std::{
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket, TcpListener}
};

/// Binds the given addresses to `UdpSocket`s
///
/// * `ipv4_addresses` - The vector of IPv4 addresses to bind to
/// * `ipv6_addresses` - The vector of IPv6 addresses to bind to
/// * `port` - The port to bind to
pub fn bind_udp_sockets(ipv4_addresses: &Vec<Ipv4Addr>, ipv6_addresses: &Vec<Ipv6Addr>, port: &u16) -> Vec<UdpSocket> {
  map_ips_to_socket_addresses(ipv4_addresses, ipv6_addresses, port)
    .iter()
    .map(|sock_addr| {
      UdpSocket::bind(sock_addr).expect(&format!("Could not bind to UDP: {}", sock_addr))
    }).collect()
}

/// Binds the given addresses to `TcpLisener`s
///
/// * `ipv4_addresses` - The vector of IPv4 addresses to bind to
/// * `ipv6_addresses` - The vector of IPv6 addresses to bind to
/// * `port` - The port to bind to
pub fn bind_tcp_listeners(ipv4_addresses: &Vec<Ipv4Addr>, ipv6_addresses: &Vec<Ipv6Addr>, port: &u16) -> Vec<TcpListener> {
  map_ips_to_socket_addresses(ipv4_addresses, ipv6_addresses, port)
    .iter()
    .map(|sock_addr| {
      TcpListener::bind(sock_addr).expect(&format!("Could not bind to TCP: {}", sock_addr))
    }).collect()
}

/// Takes IPv4 and IPv6 addresses and maps them to `SocketAddr`
///
/// This method generifies the IP version-specific information into a version-ignostic
/// _internet Socket_.
///
/// * `ipv4_addresses` - The vector of IPv4 addresses to remap
/// * `ipv6_addresses` - The vector of IPv6 addresses to remap
fn map_ips_to_socket_addresses(ipv4_addresses: &Vec<Ipv4Addr>, ipv6_addresses: &Vec<Ipv6Addr>, port: &u16) -> Vec<SocketAddr> {
  ipv4_addresses
    .into_iter()
    .map(|x| IpAddr::V4(x.clone()))
    .chain(ipv6_addresses
      .into_iter()
      .map(|x| IpAddr::V6(x.clone())))
    .flat_map(|addr| (addr, port.clone()).to_socket_addrs().unwrap())
    .collect()
}
