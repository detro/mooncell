use log::LevelFilter;
use log4rs::{
  self,
  append::console::ConsoleAppender,
  config::{Appender, Config as Log4rsConfig, Logger, Root},
};

use config::config::Config;

pub fn init(config: &Config) {
  // TODO Make the list of modules to filter out a bit more flexible
  let log_config = Log4rsConfig::builder()
    .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
    .logger(Logger::builder().build("trust_dns_proto", LevelFilter::Off))
    .build(Root::builder().appender("stdout").build(config.log_filter()))
    .unwrap();

  log4rs::init_config(log_config).unwrap();
}

pub fn init_testing() {
  let log_config = Log4rsConfig::builder()
    .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
    .logger(Logger::builder().build("trust_dns_proto", LevelFilter::Off))
    .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
    .unwrap();

  log4rs::init_config(log_config).unwrap();
}


