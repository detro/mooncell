// CLI arguments gathering
#[macro_use] extern crate clap;

// Logging
#[macro_use] extern crate log;
extern crate log4rs;

// JSON serialization/deserialization
extern crate serde_json;
#[macro_use] extern crate serde_derive;

// HTTP(S)
extern crate hyper;
extern crate hyper_tls;
extern crate url;

// DNS Protocol
extern crate trust_dns_proto;

pub mod config;
pub mod doh;
pub mod net;
pub mod logging;
pub mod dns_proto;
