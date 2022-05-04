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
  pub cipher: CipherConfig,
  pub nats: NatsConfig,
  pub proxy: ProxyConfig,
  pub telegram: TelegramConfig,
  pub bindings: DashMap<i64, ArcStr>,
  target_address_mapper: DashMap<i64, ArcStr>,
}
impl Config {
  pub fn mapper(&self, target: &i64) -> Option<ArcStr> {
    match self.bindings.get(target) {
      Some(v) => return Some(v.clone()),
      None => return None,
    }
  }
  pub fn migrate(&self){
    for pair in &self.target_address_mapper {
      self.bindings.insert(pair.key().clone(), pair.value().clone());
    }
    self.target_address_mapper.clear();
  }
}

#[basic_derive]
pub struct NatsConfig {
  // pattern: "nats://{host}:{port}"
  #[educe(Default = "nats://itsusinn.site:4222")]
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
