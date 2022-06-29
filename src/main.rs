#![allow(incomplete_features)]
#![feature(backtrace, capture_disjoint_fields)]

use futures::FutureExt;
use mesagisto_client::MesagistoConfig;
use teloxide::{prelude::*, types::ParseMode, Bot};
use tracing::{info, warn, Level};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use self::message::handlers;
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
async fn run() -> anyhow::Result<()> {
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
          // use local time
          time::UtcOffset::__from_hms_unchecked(8, 0, 0),
          time::macros::format_description!(
            "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
          ),
        )),
    )
    .with(
      tracing_subscriber::filter::Targets::new()
        .with_target("teloxide", Level::INFO)
        .with_target("telegram_message_source", Level::INFO)
        .with_target("mesagisto_client", Level::TRACE)
        .with_default(Level::WARN),
    )
    .init();

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
    .cipher_enable(CONFIG.cipher.enable)
    .cipher_key(CONFIG.cipher.key.clone())
    .cipher_refuse_plain(CONFIG.cipher.refuse_plain)
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
  info!("Mesagisto信使正在启动, version: v{}",env!("CARGO_PKG_VERSION"));

  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config())
    .parse_mode(ParseMode::MarkdownV2)
    .auto_send();

  TG_BOT.init(bot).await?;

  handlers::receive::recover()?;
  dispatch::start(&TG_BOT).await;

  CONFIG.save();
  log::info!("Mesagisto信使即将关闭");
  Ok(())
}
