//! Project configuration schema and providers
//!
//! The key component is the trait `Config`: anything that provides configuration for
//! the application to run, must implement it.
//! At the moment, only `CLI` implements it, as the configuration is provided as command line
//! interface arguments.

pub mod cli;
pub mod config;
mod defaults;