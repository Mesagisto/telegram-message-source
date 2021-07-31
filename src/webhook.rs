use log::info;
use reqwest::{StatusCode, Url};
use teloxide::{
    dispatching::{
        update_listeners::{
            self,
            StatefulListener
        },
        stop_token::AsyncStopToken
    },
    prelude::*,
    types::Update
};

use std::{convert::Infallible, env, net::SocketAddr};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;

use crate::config::CONFIG;

pub async fn webhook(bot: &AutoSend<Bot>) -> impl update_listeners::UpdateListener<Infallible> {
    // Heroku auto defines a port value
    let port: u16 = if CONFIG.telegram.webhook.heroku {
        env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer")
    } else { CONFIG.telegram.webhook.port };
    info!("The port of webhook is {}",&port);
    // Heroku host example .: "heroku-ping-pong-bot.herokuapp.com"
    let host = CONFIG.telegram.webhook.host.to_string();
    let path = format!("bot{}", CONFIG.telegram.token);
    let url = Url::parse(&format!("https://{}/{}", host, path)).unwrap();

    info!("Webhook is being setup");
    bot.set_webhook(url).await.expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            if let Ok(update) = Update::try_parse(&json) {
                tx.send(Ok(update)).expect("Cannot send an incoming update from the webhook")
            }

            StatusCode::OK
        })
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
    let server = warp::serve(server);
    let (_addr, fut) = server.bind_with_graceful_shutdown(addr, stop_flag);

    // You might want to use serve.key_path/serve.cert_path methods here to
    // setup a self-signed TLS certificate.

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S { &mut state.0 }

    StatefulListener::new((stream, stop_token), streamf, |state: &mut (_, AsyncStopToken)| state.1.clone())
}

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}