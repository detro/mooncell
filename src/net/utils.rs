use std::{
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs}
};
use tokio::net::{TcpListener, UdpSocket};

pub fn bind_udp_sockets(ipv4_addresses: &Vec<Ipv4Addr>, ipv6_addresses: &Vec<Ipv6Addr>, port: &u16) -> Vec<UdpSocket> {
  map_ip_addresses_to_sockets(ipv4_addresses, ipv6_addresses, port)
    .iter()
    .map(|sock_addr| {
      UdpSocket::bind(sock_addr).expect(&format!("Could not bind to UDP: {}", sock_addr))
    }).collect()
}

pub fn bind_tcp_listeners(ipv4_addresses: &Vec<Ipv4Addr>, ipv6_addresses: &Vec<Ipv6Addr>, port: &u16) -> Vec<TcpListener> {
  map_ip_addresses_to_sockets(ipv4_addresses, ipv6_addresses, port)
    .iter()
    .map(|sock_addr| {
      TcpListener::bind(sock_addr).expect(&format!("Could not bind to TCP: {}", sock_addr))
    }).collect()
}

fn map_ip_addresses_to_sockets(ipv4_addresses: &Vec<Ipv4Addr>, ipv6_addresses: &Vec<Ipv6Addr>, port: &u16) -> Vec<SocketAddr> {
  ipv4_addresses
    .into_iter()
    .map(|x| IpAddr::V4(x.clone()))
    .chain(ipv6_addresses
      .into_iter()
      .map(|x| IpAddr::V6(x.clone())))
    .flat_map(|addr| (addr, port.clone()).to_socket_addrs().unwrap())
    .collect()
}
