use log::LevelFilter;
use log4rs::{
  self,
  append::console::ConsoleAppender,
  config::{Appender, Config as Log4rsConfig, Logger, Root},
};

pub fn init(log_level_filter: LevelFilter) {
  // TODO Make the list of modules to filter out a bit more flexible
  let log_config = Log4rsConfig::builder()
    .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
    .logger(Logger::builder().build("mio", LevelFilter::Off))
    .logger(Logger::builder().build("tokio_core", LevelFilter::Off))
    .logger(Logger::builder().build("tokio_reactor", LevelFilter::Off))
    .logger(Logger::builder().build("tokio", LevelFilter::Off))
    .logger(Logger::builder().build("trust_dns_server", LevelFilter::Off))
    .logger(Logger::builder().build("hyper", LevelFilter::Off))
    .logger(Logger::builder().build("hyper_tls", LevelFilter::Off))
    .build(Root::builder().appender("stdout").build(log_level_filter))
    .unwrap();

  log4rs::init_config(log_config).unwrap();
}
