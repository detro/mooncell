//! Command Line Interface implementation of `Config`

use super::{defaults, config::Config};
use crate::core::{protocol::DoHProtocol, provider::DoHProvider};
use crate::doh_json::provider::DoHJsonProvider;

use clap::*;
use log::*;

use std::{net::{Ipv4Addr, Ipv6Addr}, fmt};

const ARG_IPV4: &'static str = "ipv4";
const ARG_IPV4_SHORT: &'static str = "4";
const ARG_IPV6: &'static str = "ipv6";
const ARG_IPV6_SHORT: &'static str = "6";
const ARG_PORT: &'static str = "port";
const ARG_PORT_SHORT: &'static str = "p";
const ARG_PROTOCOL: &'static str = "protocol";
const ARG_PROVIDER: &'static str = "provider";
const ARG_VERBOSE: &'static str = "verbose";
const ARG_VERBOSE_SHORT: &'static str = "v";
const ARG_QUIET: &'static str = "quiet";
const ARG_QUIET_SHORT: &'static str = "q";
const SUBCOMMAND_LIST_PROVIDERS: &'static str = "list-providers";
const SUBCOMMAND_ALIAS_LIST_PROVIDERS: &'static str = "lsprov";

/// Command Line Interface
///
/// This implements `Config` by parsing the arguments passed to the executable at launch.
#[derive(Clone)]
pub struct CLI<'a> {
  arg_matches: ArgMatches<'a>
}

impl<'a> CLI<'a> {
  /// Constructor
  ///
  /// Will automatically get the arguments received from the command line and parse them
  pub fn new() -> CLI<'a> {
    let matches = App::new(crate_name!())
      .version(crate_version!())
      .about(crate_description!())
      .author(crate_authors!("\n"))
      .arg(Arg::with_name(ARG_IPV4)
        .long(ARG_IPV4)
        .short(ARG_IPV4_SHORT)
        .required(false)
        .multiple(true)
        .default_value(defaults::IPV4_DEFAULT)
        .help("IPv4(s) to bind DNS server to")
      )
      .arg(Arg::with_name(ARG_IPV6)
        .long(ARG_IPV6)
        .short(ARG_IPV6_SHORT)
        .required(false)
        .multiple(true)
        .default_value(defaults::IPV6_DEFAULT)
        .help("IPv6(s) to bind DNS server to")
      )
      .arg(Arg::with_name(ARG_PORT)
        .long(ARG_PORT)
        .short(ARG_PORT_SHORT)
        .required(false)
        .multiple(false)
        .default_value(defaults::PORT_DEFAULT)
        .help("Port to listen on")
      )
      .arg(Arg::with_name(ARG_PROTOCOL)
        .long(ARG_PROTOCOL)
        .required(false)
        .multiple(false)
        .possible_values(&[DoHProtocol::JSON.into(), DoHProtocol::WIRE.into()])
        .default_value(DoHProtocol::JSON.into())
        .help(&format!("DoH Protocol (see subcommand '{}')", SUBCOMMAND_LIST_PROVIDERS))
      )
      .arg(Arg::with_name(ARG_PROVIDER)
        .long(ARG_PROVIDER)
        .required(false)
        .multiple(false)
        .value_name(ARG_PROVIDER)
        .help(&format!("DoH Provider (see subcommand '{}')", SUBCOMMAND_LIST_PROVIDERS))
      )
      .arg(Arg::with_name(ARG_VERBOSE)
        .long(ARG_VERBOSE)
        .short(ARG_VERBOSE_SHORT)
        .required(false)
        .multiple(true)
        .help("Verbosity level (can use multiple times)")
      )
      .arg(Arg::with_name(ARG_QUIET)
        .long(ARG_QUIET)
        .short(ARG_QUIET_SHORT)
        .required(false)
        .multiple(false)
        .help("Don't write anything to standard output (i.e. 'quiet mode')")
      )
      .subcommand(SubCommand::with_name(SUBCOMMAND_LIST_PROVIDERS)
        .visible_alias(SUBCOMMAND_ALIAS_LIST_PROVIDERS)
        .about("Available Providers, by DoH Protocol")
      )
      .get_matches();

    CLI {
      arg_matches: matches
    }
  }

  pub fn is_list_providers(&self) -> bool {
    match self.arg_matches.subcommand_name() {
      Some(SUBCOMMAND_LIST_PROVIDERS) => true,
      None | _ => false
    }
  }

  pub fn list_providers(&self) {
    println!(r#"
      protocol : providers (default)
      ============================================================ = = =
      {}     : {} ({})
      {}     : {} ({})
    "#,
             DoHProtocol::JSON, DoHJsonProvider::available_ids().join(", "), DoHJsonProvider::default_id(),
             DoHProtocol::WIRE, "NONE", "NONE"
    );
  }

}

impl<'a> Config for CLI<'a> {
  fn ipv4(&self) -> Vec<Ipv4Addr> {
    let arg_matches_ref = &self.arg_matches;
    values_t_or_exit!(arg_matches_ref, ARG_IPV4, Ipv4Addr)
  }

  fn ipv6(&self) -> Vec<Ipv6Addr> {
    let arg_matches_ref = &self.arg_matches;
    values_t_or_exit!(arg_matches_ref, ARG_IPV6, Ipv6Addr)
  }

  fn port(&self) -> u16 {
    let arg_matches_ref = &self.arg_matches;
    value_t_or_exit!(arg_matches_ref, ARG_PORT, u16)
  }

  fn log_filter(&self) -> LevelFilter {
    // Here we take 2 parameters, `quiet` and `verbose` and work out
    // how to map their use to a logging level.
    //
    // `quiet` has priority over `verbose`.
    if self.arg_matches.occurrences_of(ARG_QUIET) == 1 {
      LevelFilter::Off
    } else {
      match self.arg_matches.occurrences_of(ARG_VERBOSE) {
        0 => defaults::LOG_FILTER_DEFAULT,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace
      }
    }
  }

  fn protocol(&self) -> DoHProtocol {
    let arg_matches_ref = &self.arg_matches;
    value_t_or_exit!(arg_matches_ref, ARG_PROTOCOL, DoHProtocol)
  }

  fn provider(&self) -> Option<Box<dyn DoHProvider>> {
    match self.protocol() {
      DoHProtocol::JSON => {
        let provider_id = self.arg_matches.value_of(ARG_PROVIDER).unwrap_or(DoHJsonProvider::default_id());

        match DoHJsonProvider::available().remove(provider_id) {
          Some(provider) => Some(Box::new(provider)),
          None => {
            error!("Unknown provider '{}': see subcommand '{}'", provider_id, SUBCOMMAND_LIST_PROVIDERS);
            None
          }
        }
      },
      DoHProtocol::WIRE => {
        error!("No providers available for protocol '{}': see subcommand '{}'", DoHProtocol::WIRE, SUBCOMMAND_LIST_PROVIDERS);
        None
      },
    }
  }

}

impl<'a> fmt::Debug for CLI<'a> {
  fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
    write!(fmtr,
           "CLI (ConfigProvider) {{ ipv4: {:?}, ipv6: {:?}, port: {}, protocol: {}, provider: {:?}, log_filter: {} }}",
           self.ipv4(), self.ipv6(), self.port(), self.protocol(), self.provider(), self.log_filter())
  }
}
