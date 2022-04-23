#![allow(incomplete_features)]
#![feature(backtrace, capture_disjoint_fields)]

use futures::FutureExt;
use mesagisto_client::MesagistoConfig;
use teloxide::{prelude::*, Bot};

use crate::config::CONFIG;
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
mod message;
mod net;

#[tokio::main]
async fn main() {
  run().await.unwrap();
}
#[allow(unused_must_use)]
async fn run() -> Result<(), anyhow::Error> {
  let env = tracing_subscriber::EnvFilter::from("warn")
    .add_directive("teloxide=info".parse()?)
    .add_directive("telegram_message_source=info".parse()?)
    .add_directive("mesagisto_client=info".parse()?);
  tracing_subscriber::fmt().with_env_filter(env).init();

  if !CONFIG.enable {
    log::warn!("Mesagisto-Bot is not enabled and is about to exit the program.");
    log::warn!("To enable it, please modify the configuration file.");
    log::warn!("Mesagisto-Bot未被启用，即将退出程序。");
    log::warn!("若要启用，请修改配置文件。");
    return Ok(());
  }
  MesagistoConfig::builder()
    .name("tg")
    .cipher_enable(CONFIG.cipher.enable)
    .cipher_key(CONFIG.cipher.key.clone())
    .cipher_refuse_plain(CONFIG.cipher.refuse_plain)
    .nats_address(CONFIG.nats.address.clone())
    .proxy(if CONFIG.proxy.enable && CONFIG.proxy.enable_for_mesagisto {
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

  log::info!("Mesagisto信使正在启动");

  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config()).auto_send();

  TG_BOT.init(bot);
  dispatch::start(&TG_BOT).await;

  CONFIG.save();
  log::info!("Mesagisto信使即将关闭");
  Ok(())
}
