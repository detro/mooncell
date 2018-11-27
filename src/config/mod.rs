//! Module concerning with configuration
//!
//! The key component is the trait `ConfigProvider`: anything that provides configuration for
//! the application to run, must implement it.
//! At the moment, only `CLI` implements it, as the configuration is provided as command line
//! interface arguments.

pub mod cli;
pub mod config_provider;

mod defaults;
