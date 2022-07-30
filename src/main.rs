#![allow(incomplete_features)]
#![feature(backtrace, capture_disjoint_fields, let_chains)]

use bot::TG_BOT;
use color_eyre::eyre::Result;
use futures::FutureExt;
use mesagisto_client::MesagistoConfig;
use rust_i18n::t;
use self_update::Status;
use teloxide::{prelude::*, types::ParseMode, Bot};

use crate::config::{Config, CONFIG};

#[macro_use]
extern crate educe;
#[macro_use]
extern crate automatic_config;
#[macro_use]
extern crate singleton;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate rust_i18n;
i18n!("locales");

mod bot;
pub mod commands;
mod config;
mod dispatch;
pub mod ext;
mod handlers;
mod log;
mod net;
mod update;
mod webhook;

const TARGET: &str = "msgist";

#[tokio::main]
async fn main() -> Result<()> {
  if cfg!(feature = "color") {
    color_eyre::install()?;
  } else {
    color_eyre::config::HookBuilder::new()
      .theme(color_eyre::config::Theme::new())
      .install()?;
  }
  self::log::init().await?;
  run().await?;
  Ok(())
}

async fn run() -> Result<()> {
  Config::reload().await?;
  if !&CONFIG.locale.is_empty() {
    rust_i18n::set_locale(&CONFIG.locale);
  } else {
    use sys_locale::get_locale;
    let locale = get_locale()
      .unwrap_or_else(|| String::from("en-US"))
      .replace('_', "-");
    rust_i18n::set_locale(&locale);
    info!(
      target: TARGET,
      "{}",
      t!("log.locale-not-configured", locale_ = &locale)
    );
  }
  if !CONFIG.enable {
    warn!(target: TARGET, "{}", t!("log.not-enable"));
    warn!(target: TARGET, "{}", t!("log.not-enable-helper"));
    return Ok(());
  }
  CONFIG.migrate();

  if CONFIG.auto_update.enable {
    tokio::task::spawn_blocking(|| {
      match update::update() {
        Ok(Status::UpToDate(_)) => {
          info!(target: TARGET, "{}", t!("log.update-check-success"));
        }
        Ok(Status::Updated(_)) => {
          info!(target: TARGET, "{}", t!("log.upgrade-success"));
          std::process::exit(0);
        }
        Err(e) => {
          error!(target: TARGET, "{}", e);
        }
      };
    })
    .await?;
  }
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
    .await?;
  info!(
    target: TARGET,
    "{}",
    t!("log.boot-start", version = env!("CARGO_PKG_VERSION"))
  );
  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config())
    .parse_mode(ParseMode::Html)
    .auto_send();

  TG_BOT.init(bot).await?;

  handlers::receive::recover()?;
  tokio::spawn(async {
    dispatch::start(&TG_BOT).await;
  });
  tokio::signal::ctrl_c().await?;
  CONFIG.save().await.expect("保存配置文件失败");
  info!(target: TARGET, "{}", t!("log.shutdown"));

  #[cfg(feature = "polylith")]
  opentelemetry::global::shutdown_tracer_provider();
  Ok(())
}
