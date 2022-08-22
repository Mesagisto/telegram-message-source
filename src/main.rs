#![allow(incomplete_features)]
#![feature(capture_disjoint_fields, let_chains)]

use std::collections::HashMap;

use bot::TG_BOT;
use color_eyre::eyre::Result;
use dashmap::DashMap;
use futures::FutureExt;
use mesagisto_client::{MesagistoConfig, MesagistoConfigBuilder};
use rust_i18n::t;
use self_update::Status;
use teloxide::{prelude::*, types::ParseMode, Bot};

use crate::{
  config::{Config, CONFIG},
  handlers::receive::packet_handler,
};

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

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
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
    info!("{}", t!("log.locale-not-configured", locale_ = &locale));
  }
  if !CONFIG.enable {
    warn!("{}", t!("log.not-enable"));
    warn!("{}", t!("log.not-enable-helper"));
    return Ok(());
  }
  CONFIG.migrate();

  if cfg!(feature = "beta") {
    std::env::set_var("GH_PRE_RELEASE", "1");
    std::env::set_var("BYPASS_CHECK", "1");
  }

  if CONFIG.auto_update.enable {
    tokio::task::spawn_blocking(|| {
      match update::update() {
        Ok(Status::UpToDate(_)) => {
          info!("{}", t!("log.update-check-success"));
        }
        Ok(Status::Updated(_)) => {
          info!("{}", t!("log.upgrade-success"));
          std::process::exit(0);
        }
        Err(e) => {
          error!("{}", e);
        }
      };
    })
    .await?;
  }
  let remotes = DashMap::new();
  remotes.insert(
    arcstr::literal!("mesagisto"),
    "msgist://center.itsusinn.site:6996".into(),
  );
  MesagistoConfigBuilder::default()
    .name("tg")
    .cipher_key(CONFIG.cipher.key.clone())
    .local_address("0.0.0.0:0")
    .remote_address(remotes)
    .proxy(if CONFIG.proxy.enable {
      Some(CONFIG.proxy.address.clone())
    } else {
      None
    })
    .build()?
    .apply()
    .await?;
  MesagistoConfig::photo_url_resolver(|id_pair| {
    async {
      let file = String::from_utf8_lossy(&id_pair.1);
      let file_path = TG_BOT.get_file(file).await.unwrap().file_path;
      Ok(TG_BOT.get_url_by_path(file_path))
    }
    .boxed()
  });
  MesagistoConfig::packet_handler(|pkt| async { packet_handler(pkt).await }.boxed());
  info!(
    "{}",
    t!("log.boot-start", version = env!("CARGO_PKG_VERSION"))
  );
  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config())
    .parse_mode(ParseMode::Html)
    .auto_send();

  TG_BOT.init(bot).await?;

  handlers::receive::recover().await?;
  tokio::spawn(async {
    dispatch::start(&TG_BOT).await;
  });
  tokio::signal::ctrl_c().await?;
  CONFIG.save().await.expect("保存配置文件失败");
  info!("{}", t!("log.shutdown"));

  #[cfg(feature = "polylith")]
  opentelemetry::global::shutdown_tracer_provider();
  Ok(())
}
