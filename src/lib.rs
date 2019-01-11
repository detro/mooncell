// CLI arguments gathering
#[macro_use] extern crate clap;

// Logging
#[macro_use] extern crate log;
extern crate log4rs;

// JSON serialization/deserialization
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

// DNS Protocol
extern crate trust_dns_proto;

// HTTP requests/response
extern crate http;

// IP Network addresses manipulation
extern crate ipnet;

pub mod config;
pub mod dns;
pub mod doh_json;
pub mod doh_wire;
pub mod net;
pub mod logging;
pub mod core;
