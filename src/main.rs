use std::env;
use std::sync::Arc;
use teloxide::{prelude::*, Bot};
use tokio::signal;
use std::{
    collections::{HashMap, HashSet}
};
use nats::Headers;

#[macro_use]
extern crate lazy_static;

pub mod bot;
pub mod command;
pub mod config;
pub mod cover;
pub mod data;
mod message;

use config::CONFIG;
use data::DATA;


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    run().await
}

async fn run() -> Result<(), anyhow::Error> {

    teloxide::enable_logging!();

    if !CONFIG.enabled {
        log::info!("Mesagisto-Bot is not enabled");
        return Ok(());
    }

    log::info!("Mesagisto-Bot started");

    let opts = async_nats::Options::new();

    let nc = opts
        .with_name("telegram client")
        .connect(&CONFIG.forwarding.address).await?;

    let cid = nc.client_id().to_string();
    let nats_header = {
        let mut inner = HashMap::default();
        let entry = inner.entry("cid".to_string()).or_insert_with(HashSet::default);
        let clone_cid = cid.clone();
        entry.insert(clone_cid);
        Arc::new(Headers { inner })
    };

    if CONFIG.proxy.enabled {
        env::set_var("TELOXIDE_PROXY", &CONFIG.proxy.address);
    }

    let bot = Bot::with_client(
        CONFIG.telegram.token,
        teloxide::net::client_from_env()
    ).auto_send();

    let clone_header = nats_header.clone();
    let clone_cid = Arc::new(cid);
    let clone_bot = Arc::new(bot.clone());
    let clone_nc = nc.clone();

    tokio::spawn(async move {
        bot::commands_or_message_repl(
            bot,
            &*CONFIG.telegram.bot_name,
            command::answer,
            move |cx,msg| {

                let clone_header = clone_header.clone();
                let clone_cid = clone_cid.clone();
                let clone_nc = clone_nc.clone();
                let clone_bot = clone_bot.clone();

                async move {
                    if !message::answer_msg(cx.clone(), &msg).await? {
                        let target = cx.chat_id();
                        if CONFIG.target_address_mapper.contains_key(&target) {
                            let address = *CONFIG.target_address_mapper.get(&target).unwrap();
                            let content = format!("{}: {}", cx.update.from().unwrap().username.to_owned().unwrap(),msg);
                            clone_nc.publish_with_reply_or_headers(
                                address,
                                None,
                                Some(&*clone_header),
                                content).await.unwrap();

                            if  !DATA.active_endpoint.contains_key(&target) {
                                DATA.active_endpoint.insert(target, true);

                                let sub = clone_nc.subscribe(address).await.unwrap();
                                tokio::spawn( async move  {
                                    let clone_bot = clone_bot.clone();
                                    loop {
                                        if let Some(msg) =  sub.next().await {
                                            if let Some(headers) =  msg.headers{
                                                if let Some(cid) = headers.get("cid"){
                                                    if cid.contains(&*clone_cid) {
                                                        continue;
                                                    }
                                                }
                                            }
                                            let data = msg.data;
                                            clone_bot.send_message(target, String::from_utf8_lossy(&data)).await.unwrap();
                                        }
                                    }
                                });
                            }
                        }
                    };
                    respond(())
                }
            },
        ).await;
    });

    signal::ctrl_c().await.expect("Failed to listen ctrl+c event");


    CONFIG.save();
    log::info!("Mesagisto Bot is going to shut down");
    Ok(())
}
