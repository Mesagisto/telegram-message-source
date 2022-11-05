use color_eyre::eyre::Result;
use self_update::{cargo_crate_version, Status};

use crate::config::CONFIG;

const SHORT_NAME: &str = "tg";

pub fn update() -> Result<Status> {
  if CONFIG.auto_update.enable_proxy && CONFIG.proxy.enable {
    std::env::set_var("HTTPS_PROXY", CONFIG.proxy.address.as_str());
  }
  let status = self_update::backends::github::Update::configure()
    .repo_owner("MeowCat-Studio")
    .repo_name("telegram-message-source")
    .bin_name(&bin_name(SHORT_NAME))
    .show_download_progress(true)
    .target(&target_name())
    .current_version(current_version())
    .no_confirm(CONFIG.auto_update.no_confirm)
    .build()?
    .update()?;
  Ok(status)
}

fn current_version() -> &'static str {
  if cfg!(feature = "beta") {
    option_env!("MESAGISTO_VERSION").unwrap_or_else(|| cargo_crate_version!())
  } else {
    cargo_crate_version!()
  }
}

fn bin_name(short: &str) -> String {
  if cfg!(target_os = "windows") {
    format!("{short}-{}.exe", target_name())
  } else {
    format!("{short}-{}", target_name())
  }
}

#[test]
fn test() -> Result<()> {
  // update()
  println!("target-name: {}", target_name());
  println!("bin-name: {}", bin_name(SHORT_NAME));
  Ok(())
}

fn target_name() -> String {
  let arch = if cfg!(target_arch = "x86_64") {
    "x86_64"
  } else if cfg!(target_arch = "i686") {
    "i686"
  } else if cfg!(target_arch = "aarch64") {
    "aarch64"
  } else {
    "unknown"
  };
  let os = if cfg!(target_os = "linux") {
    "linux"
  } else if cfg!(target_os = "windows") && cfg!(feature = "color") {
    "windows-colored"
  } else if cfg!(target_os = "windows") && cfg!(target_os = "no-color") {
    "windows"
  } else if cfg!(target_os = "darwin") {
    "mac"
  } else {
    "unknown"
  };
  format!("{arch}-{os}")
}

#[test]
fn test_reqwest() -> Result<()> {
  use std::time::Duration;

  use reqwest::header;
  let a = reqwest::blocking::Client::builder()
    .user_agent("rust-reqwest/self-update")
    .timeout(Duration::from_secs(5))
    // .proxy(Proxy::http("127.0.0.1:7890").unwrap())
    .build()
    .unwrap()
    .post("https://api.github.com/repos/MeowCat-Studio/telegram-message-source/releases/assets/72134624")
    .header(header::ACCEPT, "application/octet-stream")
    .send()?;
  dbg!(a);
  Ok(())
}
