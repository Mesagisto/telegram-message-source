use std::convert::TryInto;

use crate::ext::db::DbExt;
use crate::CONFIG;
use crate::TG_BOT;
use mesagisto_client::db::DB;
use mesagisto_client::{
  cache::CACHE,
  data::{message::Message, message::MessageType, Packet},
};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

use teloxide::types::InputFile;

pub async fn receive_from_server(message: nats::asynk::Message, target: Vec<u8>) -> anyhow::Result<()> {
  let target = i64::from_be_bytes(target.try_into().unwrap());
  log::trace!("接收到来自目标{}的消息", target);
  let packet = Packet::from_cbor(&message.data)?;
  match packet {
    either::Left(msg) => {
      handle_receive_message(msg, target).await?;
    }
    either::Right(_) => {}
  }
  Ok(())
}

pub async fn handle_receive_message(mut message: Message, target: i64) -> anyhow::Result<()> {
  for single in message.chain {
    log::trace!("正在处理消息链中的元素");
    let sender_name = if message.profile.nick.is_some() {
      message.profile.nick.take().unwrap()
    } else if message.profile.username.is_some() {
      message.profile.username.take().unwrap()
    } else {
      base64_url::encode(&message.profile.id)
    };
    match single {
      MessageType::Text { content } => {
        let content = format!("{}:\n{}", sender_name, content);
        let receipt = if let Some(reply_to) = &message.reply {
          let local_id = DB.get_msg_id_1(&target, reply_to)?;
          match local_id {
            Some(local_id) => {
              TG_BOT
                .send_message(target, content)
                .reply_to_message_id(local_id)
                .await?
            }
            None => TG_BOT.send_message(target, content).await?,
          }
        } else {
          TG_BOT.send_message(target, content).await?
        };
        DB.put_msg_id_1(&target, &message.id, &receipt.id)?;
      }
      MessageType::Image { id, url } => {
        let channel = CONFIG.mapper(&target).expect("频道不存在");
        let path = CACHE.file(&id, &url, &channel).await?;
        let receipt = TG_BOT
          .send_message(target, format!("{}:", sender_name))
          .await?;
        DB.put_msg_id_ir_2(&target, &receipt.id, &message.id)?;
        let receipt = TG_BOT.send_photo(target, InputFile::File(path)).await?;
        DB.put_msg_id_1(&target, &message.id, &receipt.id)?;
      }
    }
  }

  Ok(())
}
