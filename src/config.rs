use arcstr::ArcStr;
use color_eyre::eyre::{Error, Result};
use dashmap::DashMap;

#[config_derive]
#[derive(AutomaticConfig)]
#[location = "config/tg.yml"]
pub struct Config {
  #[educe(Default = false)]
  pub enable: bool,
  #[educe(Default = "")]
  pub locale: ArcStr,
  // A-z order
  pub bindings: DashMap<i64, ArcStr>,
  pub cipher: CipherConfig,
  pub nats: NatsConfig,
  pub proxy: ProxyConfig,
  pub telegram: TelegramConfig,
  pub auto_update: AutoUpdateConfig,
  // TODO remove in next major version
  target_address_mapper: DashMap<i64, ArcStr>,
}

impl Config {
  pub fn mapper(&self, target: &i64) -> Option<ArcStr> {
    self.bindings.get(target).map(|v| v.clone())
  }

  pub fn migrate(&self) {
    for pair in &self.target_address_mapper {
      self.bindings.insert(*pair.key(), pair.value().clone());
    }
    self.target_address_mapper.clear();
  }

  pub fn migrate_chat(&self, old_chat_id: &i64, new_chat_id: &i64) -> Option<ArcStr> {
    if let Some((_, address)) = self.bindings.remove(old_chat_id) {
      self.bindings.insert(*new_chat_id, address.clone());
      return Some(address);
    };
    None
  }
}

#[config_derive]
pub struct NatsConfig {
  // pattern: "nats://{host}:{port}"
  #[educe(Default = "nats://nats.mesagisto.org:4222")]
  pub address: ArcStr,
}

#[config_derive]
pub struct ProxyConfig {
  #[educe(Default = false)]
  pub enable: bool,
  // pattern: "http://{username}:{password}@{host}:{port}"
  #[educe(Default = "http://127.0.0.1:7890")]
  pub address: ArcStr,
}

#[config_derive]
pub struct CipherConfig {
  #[educe(Default = "default")]
  pub key: ArcStr,
}

#[config_derive]
pub struct TelegramConfig {
  #[educe(Default = "BOT_TOKEN")]
  pub token: String,
}

#[config_derive]
pub struct FormatConfig {
  pub msg: ArcStr,
}

#[config_derive]
pub struct AutoUpdateConfig {
  #[educe(Default = true)]
  pub enable: bool,
  #[educe(Default = true)]
  pub enable_proxy: bool,
  #[educe(Default = false)]
  pub no_confirm: bool,
}
