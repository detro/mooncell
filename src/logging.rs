use log::LevelFilter;
use log4rs::{
  self,
  append::console::ConsoleAppender,
  config::{Appender, Config as Log4rsConfig, Logger, Root},
};

use config::config_provider::ConfigProvider;

pub fn init<C: ConfigProvider>(config: &C) {
  // TODO Make the list of modules to filter out a bit more flexible
  let log_config = Log4rsConfig::builder()
    .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
//    .logger(Logger::builder().build("mio", LevelFilter::Off))
//    .logger(Logger::builder().build("tokio_core", LevelFilter::Off))
//    .logger(Logger::builder().build("tokio_reactor", LevelFilter::Off))
//    .logger(Logger::builder().build("tokio", LevelFilter::Off))
//    .logger(Logger::builder().build("trust_dns_server", LevelFilter::Off))
//    .logger(Logger::builder().build("hyper", LevelFilter::Off))
//    .logger(Logger::builder().build("hyper_tls", LevelFilter::Off))
    .build(Root::builder().appender("stdout").build(config.log_filter()))
    .unwrap();

  log4rs::init_config(log_config).unwrap();
}
