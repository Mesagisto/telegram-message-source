use arcstr::ArcStr;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[basic_derive]
#[derive(AutoConfig)]
#[location = "config/tg.yml"]
pub struct Config {
  #[educe(Default = false)]
  pub enable: bool,
  // A-z order
  pub bindings: DashMap<i64, ArcStr>,
  pub cipher: CipherConfig,
  pub nats: NatsConfig,
  pub proxy: ProxyConfig,
  pub telegram: TelegramConfig,
  target_address_mapper: DashMap<i64, ArcStr>,
}

impl Config {
  pub fn mapper(&self, target: &i64) -> Option<ArcStr> {
    match self.bindings.get(target) {
      Some(v) => return Some(v.clone()),
      None => return None,
    }
  }
  pub fn migrate(&self) {
    for pair in &self.target_address_mapper {
      self
        .bindings
        .insert(pair.key().clone(), pair.value().clone());
    }
    self.target_address_mapper.clear();
  }
  pub fn migrate_chat(&self, old_chat_id: &i64, new_chat_id: &i64) -> Option<ArcStr> {
    if let Some((_, address)) = self.bindings.remove(&old_chat_id) {
      self.bindings.insert(*new_chat_id, address.clone());
      return Some(address);
    };
    return None;
  }
}

#[basic_derive]
pub struct NatsConfig {
  // pattern: "nats://{host}:{port}"
  #[educe(Default = "nats://nats.mesagisto.org:4222")]
  pub address: ArcStr,
}

#[basic_derive]
pub struct ProxyConfig {
  #[educe(Default = false)]
  pub enable: bool,
  // pattern: "http://{username}:{password}@{host}:{port}"
  #[educe(Default = "http://127.0.0.1:7890")]
  pub address: ArcStr,
}

#[basic_derive]
pub struct CipherConfig {
  #[educe(Default = true)]
  pub enable: bool,
  #[educe(Default = "this-is-an-example-key")]
  pub key: ArcStr,
  #[educe(Default = true)]
  pub refuse_plain: bool,
}

#[basic_derive]
pub struct TelegramConfig {
  #[educe(Default = "BOT_TOKEN")]
  pub token: String,
}

#[basic_derive]
pub struct FormatConfig {
  pub msg: ArcStr,
}
