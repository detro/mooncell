#[macro_use] extern crate log;
extern crate log4rs;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate tokio;
extern crate hyper;
extern crate hyper_tls;
extern crate url;
extern crate trust_dns_server;
extern crate trust_dns_proto;

pub mod core;
pub mod doh;
pub mod net;
pub mod logging;
