#![allow(incomplete_features)]
#![feature(backtrace, capture_disjoint_fields, let_chains)]

use futures::FutureExt;
use mesagisto_client::MesagistoConfig;
use teloxide::{prelude::*, types::ParseMode, Bot};
use tracing::*;

use self::message::handlers;
use crate::config::{Config, CONFIG};
use bot::TG_BOT;

#[macro_use]
extern crate educe;
#[macro_use]
extern crate automatic_config;
#[macro_use]
extern crate singleton;
mod bot;
mod command;
mod config;
mod dispatch;
pub mod ext;
mod log;
mod message;
mod net;

#[tokio::main]
async fn main() {
  self::log::init();
  run().await.unwrap();
}
#[allow(unused_must_use)]
async fn run() -> anyhow::Result<()> {
  Config::reload().await?;
  if !CONFIG.enable {
    warn!("Mesagisto-Bot is not enabled and is about to exit the program.");
    warn!("To enable it, please modify the configuration file.");
    warn!("Mesagisto-Bot未被启用, 即将退出程序。");
    warn!("若要启用，请修改配置文件。");
    return Ok(());
  }
  CONFIG.migrate();

  MesagistoConfig::builder()
    .name("tg")
    .cipher_key(CONFIG.cipher.key.clone())
    .nats_address(CONFIG.nats.address.clone())
    .proxy(if CONFIG.proxy.enable {
      Some(CONFIG.proxy.address.clone())
    } else {
      None
    })
    .photo_url_resolver(|id_pair| {
      async {
        let file = String::from_utf8_lossy(&id_pair.1);
        let file_path = TG_BOT.get_file(file).await.unwrap().file_path;
        Ok(TG_BOT.get_url_by_path(file_path))
      }
      .boxed()
    })
    .build()
    .apply()
    .await;
  info!(
    "Mesagisto信使正在启动, version: v{}",
    env!("CARGO_PKG_VERSION")
  );

  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config())
    .parse_mode(ParseMode::Html)
    .auto_send();

  TG_BOT.init(bot).await?;

  handlers::receive::recover()?;
  dispatch::start(&TG_BOT).await;

  CONFIG.save().await.expect("保存配置文件失败");
  info!("Mesagisto信使即将关闭");
  Ok(())
}
