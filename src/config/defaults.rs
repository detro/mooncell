//! Default configuration values
//!
//! This is here to keep things DRY

use log::LevelFilter;

pub const IPV4_DEFAULT: &'static str = "127.0.0.1";
pub const IPV6_DEFAULT: &'static str = "::1";
pub const PORT_DEFAULT: &'static str = "53";
pub const LOG_FILTER_DEFAULT: LevelFilter = LevelFilter::Error;