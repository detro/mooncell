use std::time::Duration;
use tokio::{
  net::{TcpListener, UdpSocket},
  prelude::{Future, future},
  runtime::Runtime,
};
use trust_dns_server::server::{RequestHandler, ServerFuture};

pub fn listen<T: RequestHandler + Send + 'static>(
  udp_sockets: Vec<UdpSocket>,
  tcp_listeners: Vec<TcpListener>,
  request_handler: T) {

  let mut tokio_runtime: Runtime = Runtime::new().expect("Unable to launch internal Tokio Runtime");
  let trust_dns_server: ServerFuture<T> = ServerFuture::new(request_handler);

  let server_future: Box<Future<Item=(), Error=()> + Send> = Box::new(future::lazy(move || {
    // load all the listeners
    for udp_socket in udp_sockets {
      info!("Listening for UDP requests on {:?}", udp_socket);
      trust_dns_server.register_socket(udp_socket);
    }

    // and TCP as necessary
    for tcp_listener in tcp_listeners {
      info!("Listening for TCP requests on {:?}", tcp_listener);
      trust_dns_server
        .register_listener(tcp_listener, Duration::from_secs(5))
        .expect("could not register TCP listener");
    }

//      let tls_cert_config = config.get_tls_cert().clone();
//
//      // and TLS as necessary
//      if let Some(tls_cert_config) = tls_cert_config {
//        config_tls(
//          &args,
//          &mut server,
//          &config,
//          tls_cert_config,
//          zone_dir,
//          &listen_addrs,
//        );
//      }
    future::empty()
  }));

  tokio_runtime.spawn(server_future.map_err(|e| error!("{:?}", e)));

  tokio_runtime.shutdown_on_idle().wait().unwrap();
}



