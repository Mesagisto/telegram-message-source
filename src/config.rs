use arcstr::ArcStr;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[basic_derive]
#[derive(AutoConfig)]
#[location = "config/tg.yml"]
pub struct Config {
  #[educe(Default = false)]
  pub enabled: bool,
  pub nats: NatsConfig,
  pub telegram: TelegramConfig,
  pub proxy: ProxyConfig,
  pub target_address_mapper: DashMap<i64, ArcStr>,
}

impl Config {
  pub fn mapper(&self,target: &i64) -> Option<ArcStr>{
    match self.target_address_mapper.get(target) {
        Some(v) => return Some(v.clone()),
        None => return None,
    }
  }
}

#[basic_derive]
pub struct NatsConfig {
  // pattern: "nats://{host}:{port}"
  #[educe(Default = "nats://itsusinn.site:4222")]
  pub address: String,
}

#[basic_derive]
pub struct ProxyConfig {
  #[educe(Default = false)]
  pub enabled: bool,
  // pattern: "http://{username}:{password}@{host}:{port}"
  #[educe(Default = "http://127.0.0.1:7890")]
  pub address: String,
}

#[basic_derive]
pub struct CipherConfig {
  #[educe(Default = false)]
  pub enabled: bool,
  // pattern: "http://{username}:{password}@{host}:{port}"
  #[educe(Default = "this-is-an-example-key")]
  pub key: String,
}

#[basic_derive]
pub struct TelegramConfig {
  #[educe(Default = "BOT_TOKEN")]
  pub token: String,
  #[educe(Default = "BOT_NAME")]
  pub bot_name: String,
  pub webhook: WebhookConfig,
}

#[basic_derive]
pub struct WebhookConfig {
  #[educe(Default = false)]
  pub enable: bool,
  #[educe(Default = false)]
  pub heroku: bool,
  #[educe(Default = 8889)]
  pub port: u16,
  #[educe(Default = "heroku-app-name.herokuapp.com")]
  pub host: String,
}

#[basic_derive]
pub struct BehaviorConfig {}

#[basic_derive]
pub struct FormatConfig {}
