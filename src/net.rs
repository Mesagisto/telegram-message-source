use std::time::Duration;

fn default_reqwest_settings() -> reqwest::ClientBuilder {
  let connect_timeout = Duration::from_secs(10);
  let timeout = connect_timeout + Duration::from_secs(24);

  reqwest::Client::builder()
    .connect_timeout(connect_timeout)
    .timeout(timeout)
}
pub fn client_from_config() -> reqwest::Client {
  use crate::config::CONFIG;
  let builder = default_reqwest_settings().use_rustls_tls();
  if CONFIG.proxy.enable {
    builder.proxy(
      reqwest::Proxy::all(CONFIG.proxy.address.as_str()).expect("reqwest::Proxy creation failed"),
    )
  } else {
    builder
  }
  .build()
  .expect("creating reqwest::Client")
}
