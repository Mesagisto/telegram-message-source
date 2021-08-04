use log::error;
use serde_derive::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};


lazy_static! {
    pub static ref CONFIG: Config = {
        let path = Path::new("config.toml");
        let config = read_or_create_config(path).unwrap();
        config.transfer();
        config
    };
}

#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct Config {
    #[educe(Default = false)]
    pub enabled: bool,
    pub forwarding: ForwardingConfig,
    pub telegram: TelegramConfig,
    pub proxy: ProxyConfig,
    #[serde(skip)]
    pub target_address_mapper: DashMap<i64, &'static str>,
    #[serde(alias = "target_address_mapper")]
    target_address_mapper_storage: DashMap<&'static str, &'static str>,
    #[serde(skip, default = "default_config_path")]
    #[educe(Default(expression = "default_config_path()"))]
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct ForwardingConfig {
    // pattern: "nats://{host}:{port}"
    #[educe(Default = "nats://itsusinn.site:4222")]
    pub address: &'static str
}

#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct ProxyConfig {
    #[educe(Default = false)]
    pub enabled: bool,
    // pattern: "http://{username}:{password}@{host}:{port}"
    #[educe(Default = "http://127.0.0.1:7890")]
    pub address: &'static str,
}

#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct TelegramConfig {
    #[educe(Default = "BOT_TOKEN")]
    pub token: &'static str,
    #[educe(Default = "BOT_NAME")]
    pub bot_name: &'static str,
    pub webhook: WebhookConfig
}

#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct WebhookConfig {
    #[educe(Default = false)]
    pub enable: bool,
    #[educe(Default = false)]
    pub heroku:bool,
    #[educe(Default = 8889)]
    pub port:u16,
    #[educe(Default = "heroku-app-name.herokuapp.com")]
    pub host:&'static str
}

impl Config {
    pub fn default_string() -> Result<String, Error> {
        let result = toml::to_string_pretty(&Config::default())
            .map_err(|_| Error::SerializationError)?;
        Ok(result)
    }
    pub fn save(&self) {
        for pair in self.target_address_mapper.iter(){
            let key = pair.key();
            let val = pair.value();
            self.target_address_mapper_storage.insert(
                Box::leak(key.to_string().into_boxed_str()),
                val
            );
        }

        let ser = toml::ser::to_string_pretty(self).unwrap();
        log::info!("Configuration file was saved");
        fs::write(self.config_path.as_path(), ser).unwrap();
    }
    fn transfer(&self){
        for pair in self.target_address_mapper_storage.iter() {
            let key = pair.key();
            let val = pair.value();
            self.target_address_mapper.insert(key.parse().unwrap(), val);
        }
        self.target_address_mapper_storage.clear();
        self.target_address_mapper_storage.shrink_to_fit();

    }
}

fn default_config_path() -> PathBuf {
    return Path::new("config.toml").to_owned();
}

fn read_or_create_config(path: &Path) -> Result<Config, Error> {
    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap_or(Path::new("./")))?;
        fs::write(path, Config::default_string()?)?;
    };
    let data = fs::read(path)?;
    let result: Result<Config, toml::de::Error> = toml::from_slice(Box::leak(data.into_boxed_slice()));
    let mut result = match result {
        Ok(val) => val,
        Err(_) => {
            error!("Cannot de-serialize the configuration file");
            error!("It may be caused by incompatible configuration files due to version updates");
            error!("The original file has been changed to config.toml.old, please merge the configuration files manually");
            let default_string = Config::default_string()?;
            let reanme_path = format!("{}.old", path.clone().to_string_lossy());
            let rename_path =Path::new(&reanme_path);
            fs::rename(path, rename_path)?;
            fs::write(path, default_string)?;
            Config::default()
        }
    };
    result.config_path = path.to_owned();
    Ok(result)
}

use dashmap::DashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("无法序列化")]
    SerializationError,
    #[error("I/O error")]
    IO(#[from] io::Error),
}
