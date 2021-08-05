
fn default_reqwest_settings() -> reqwest::ClientBuilder {
    teloxide_core::net::default_reqwest_settings()
}
pub fn client_from_config() -> reqwest::Client {
    use crate::config::CONFIG;
    let builder = default_reqwest_settings().use_rustls_tls();
    if CONFIG.proxy.enabled {
        builder.proxy(reqwest::Proxy::all(&CONFIG.proxy.address).expect("reqwest::Proxy creation failed"))
    } else {
        builder
    }
    .build()
    .expect("creating reqwest::Client")
}