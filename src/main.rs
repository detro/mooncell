#[macro_use] extern crate log;

extern crate mooncell;

use mooncell::{logging, net, config::{cli::CLI, config_provider::ConfigProvider}};

fn main() {
  let cli = CLI::new();
  println!("{:#?}", cli);

  logging::init(cli.log_filter());
  info!("DNS Server starting");

//  // Establish addresses we will listen on
//  let listen_ipv4_addr: Vec<Ipv4Addr> = vec![Ipv4Addr::from_str("0.0.0.0").unwrap()];
//  let listen_ipv6_addr: Vec<Ipv6Addr> = vec![Ipv6Addr::from_str("::").unwrap()];
//  let listen_port: u16 = 1053;

//  // Binding UDP sockets
//  let udp_sockets: Vec<UdpSocket> = net::utils::bind_udp_sockets(&listen_ipv4_addr, &listen_ipv6_addr, &listen_port);
//  debug!("Bound: {:?}", udp_sockets);
//
//  // Binding TCP sockets
//  let tcp_listeners: Vec<TcpListener> = net::utils::bind_tcp_listeners(&listen_ipv4_addr, &listen_ipv6_addr, &listen_port);
//  debug!("Bound: {:?}", tcp_listeners);
//
//  net::server_old::listen(udp_sockets, tcp_listeners, core::dns_query_handler::DnsQueryHandler::new());
}
