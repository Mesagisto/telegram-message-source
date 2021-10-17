#![allow(incomplete_features)]
#![feature(backtrace,capture_disjoint_fields)]

use std::sync::Arc;

use arcstr::ArcStr;
use futures::FutureExt;
use log::{info, warn};
use mesagisto_client::{OptionExt, cache::CACHE, cipher::CIPHER, db::DB, res::RES, server::SERVER};
use teloxide::{Bot, prelude::*};

use bot::TG_BOT;
use config::CONFIG;
use despatch::cmd_or_msg_repl;

#[macro_use]
extern crate educe;
#[macro_use]
extern crate automatic_config;
#[macro_use]
extern crate singleton;
mod bot;
mod command;
mod config;
mod data;
mod despatch;
mod message;
mod net;
mod webhook;

fn main() {
  std::env::set_var("RUST_BACKTRACE", "1");
  std::backtrace::Backtrace::force_capture();
  env_logger::builder()
    .write_style(env_logger::WriteStyle::Auto)
    .filter(None, log::LevelFilter::Error)
    .format_timestamp(None)
    .filter(Some("telegram_mesaga_fonto"), log::LevelFilter::Trace)
    .filter(Some("mesagisto_client"), log::LevelFilter::Trace)
    .filter(Some("teloxide"), log::LevelFilter::Info)
    .init();
  tokio::runtime::Builder::new_multi_thread()
  // fixme: how many do we need
    .worker_threads(5)
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      run().await.unwrap();
    });
}
#[allow(unused_must_use)]
async fn run() -> Result<(), anyhow::Error> {
  if !CONFIG.enabled {
    warn!("Mesagisto-Bot is not enabled and is about to exit the program.");
    warn!("To enable it, please modify the configuration file.");
    warn!("Mesagisto-Bot未被启用，即将退出程序。");
    warn!("若要启用，请修改配置文件。");
    return Ok(());
  }
  CIPHER.init(&"this is an example key".to_string());
  info!("Mesagisto-Bot is starting up");
  info!("Mesagisto-Bot正在启动");
  DB.init(ArcStr::from("tg").some());
  RES.init().await;
  RES.resolve_photo_url(|id_pair| {
    async {
      let file_path = TG_BOT.get_file(id_pair.1.as_str()).await.unwrap().file_path;
      Ok(TG_BOT.get_url_by_path(file_path))
    }.boxed()
  });
  SERVER.init(&CONFIG.nats.address).await;
  CACHE.init();
  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config()).auto_send();

  TG_BOT.init(Arc::new(bot));

  cmd_or_msg_repl(
    &*TG_BOT,
    &*CONFIG.telegram.bot_name,
    command::answer,
    message::handler::answer_msg,
  ).await;

  CONFIG.save();
  log::info!("Mesagisto Bot is going to shut down");
  Ok(())
}
