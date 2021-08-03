use std::env;
use std::sync::Arc;
use async_nats::Connection;
use teloxide::{prelude::*, Bot};
use std::{
    collections::{HashMap, HashSet}
};
use nats::Headers;

#[macro_use]
extern crate lazy_static;

mod command;
mod config;
mod data;
mod message;
mod webhook;
mod despatch;

use config::CONFIG;
use data::DATA;
use despatch::cmd_or_msg_repl;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    run().await
}

async fn run() -> Result<(), anyhow::Error> {

    teloxide::enable_logging!();

    if !CONFIG.enabled {
        log::info!("Mesagisto-Bot is not enabled and is about to exit the program");
        return Ok(());
    }

    log::info!("Mesagisto-Bot is starting up");

    let opts = async_nats::Options::new();

    log::info!("Connecting to nats server");
    let nc = opts
        .with_name("telegram client")
        .connect(&CONFIG.forwarding.address).await?;

    let cid = nc.client_id().to_string();
    log::info!("Connected sucessfully,the client id is {}",&cid);
    let nats_header = {
        let mut inner = HashMap::default();
        let entry = inner.entry("cid".to_string()).or_insert_with(HashSet::default);
        entry.insert(cid.clone());
        Arc::new(Headers { inner })
    };

    if CONFIG.proxy.enabled {
        log::info!("Proxy will be enable for teloxide");
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

    cmd_or_msg_repl(
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
                        let sender = cx.update.from().unwrap();
                        let sender_name = if sender.username.is_none() {
                            sender.full_name().replace(|c: char| !c.is_alphanumeric(),"")
                        } else {
                            sender.username.clone().unwrap()
                        };
                        let content = format!("{}: {}", sender_name,msg);
                        clone_nc.publish_with_reply_or_headers(
                            address,
                            None,
                            Some(&*clone_header),
                            content).await.unwrap();
                        try_create_endpoint(&clone_nc,target, address,clone_cid,clone_bot).await;
                    }
                };
                respond(())
            }
        },
    ).await;

    CONFIG.save();
    log::info!("Mesagisto Bot is going to shut down");
    Ok(())
}

async fn try_create_endpoint(nc:&Connection,target:i64,address:&str,cid:Arc<String>,bot:Arc<AutoSend<Bot>>){
    log::info!("Trying to create sub for {}",target);
    if  !DATA.active_endpoint.contains_key(&target) {
        DATA.active_endpoint.insert(target, true);
        log::info!("Creating sub for {}",target);
        let sub = nc.subscribe(address).await.unwrap();
        tokio::spawn( async move  {
            loop {
                let next = sub.next().await;
                if next.is_none() { continue; }
                let next = next.unwrap();

                let headers = next.headers;
                if headers.is_none() { continue; }
                let headers = headers.unwrap();

                let cid_set = headers.get("cid");
                if cid_set.is_none() {continue;}

                if cid_set.unwrap().contains(&*cid) {continue;}

                if let Err(err) = bot.send_message(
                    target, String::from_utf8_lossy(&next.data)
                ).await{
                    log::error!("Teloxide error {}",&err);
                }
            }
        });
    }
}
