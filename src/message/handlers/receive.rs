use crate::ext::db::DbExt;
use crate::CONFIG;
use crate::TG_BOT;
use arcstr::ArcStr;
use lateinit::LateInit;
use mesagisto_client::{
  cache::CACHE,
  data::{message::Message, message::MessageType, Packet},
  db::DB,
  server::SERVER,
};

use teloxide::types::ChatId;
use teloxide::utils::html;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;
use tracing::trace;

static CHANNEL: LateInit<UnboundedSender<(i64, ArcStr)>> = LateInit::new();

pub fn recover() -> anyhow::Result<()> {
  let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(i64, ArcStr)>();
  tokio::spawn(async move {
    while let Some(element) = rx.recv().await {
      let res = SERVER
        .recv(
          ArcStr::from(element.0.to_string()),
          &element.1,
          server_msg_handler,
        )
        .await;
      match res {
        Ok(_) => {}
        Err(e) => {
          error!("error when add callback handler {}\n{}", e, e.backtrace());
        }
      }
    }
  });
  for pair in &CONFIG.bindings {
    tx.send((pair.key().to_owned(), pair.value().clone()))?;
  }
  CHANNEL.init(tx);
  Ok(())
}

pub fn add(target: i64, address: &ArcStr) -> anyhow::Result<()> {
  CHANNEL.send((target, address.clone()))?;
  Ok(())
}
pub fn change(target: i64, address: &ArcStr) -> anyhow::Result<()> {
  SERVER.unsub(&target.to_string().into());
  add(target, address)?;
  Ok(())
}
pub fn del(target: i64) -> anyhow::Result<()> {
  SERVER.unsub(&target.to_string().into());
  Ok(())
}
pub async fn server_msg_handler(
  message: nats::asynk::Message,
  target: ArcStr,
) -> anyhow::Result<()> {
  let target: i64 = target.parse()?;
  trace!("接收到来自目标{}的消息", target);
  let packet = Packet::from_cbor(&message.data);
  let packet = match packet {
    Ok(v) => v,
    Err(_e) => {
      //todo logging
      tracing::warn!("未知的数据包类型，请更新本消息源，若已是最新请等待适配");
      return Ok(());
    }
  };
  match packet {
    either::Left(msg) => {
      left_sub_handler(msg, target).await?;
    }
    either::Right(_) => {}
  }
  Ok(())
}

async fn left_sub_handler(mut message: Message, target: i64) -> anyhow::Result<()> {
  let chat_id = ChatId(target);
  let sender_name = if message.profile.nick.is_some() {
    message.profile.nick.take().unwrap()
  } else if message.profile.username.is_some() {
    message.profile.username.take().unwrap()
  } else {
    base64_url::encode(&message.profile.id)
  };

  for single in message.chain {
    trace!("正在处理消息链中的元素");
    match single {
      MessageType::Text { content } => {
        let content = format!(
          "{}:\n{}",
          html::bold(sender_name.as_str()),
          html::escape(content.as_str())
        );
        let receipt = if let Some(reply_to) = &message.reply {
          let local_id = DB.get_msg_id_1(&target, reply_to)?;
          TG_BOT.send_text(chat_id, content, local_id).await?
        } else {
          TG_BOT.send_text(chat_id, content, None).await?
        };
        DB.put_msg_id_1(&target, &message.id, &receipt.id)?;
      }
      MessageType::Image { id, url } => {
        let channel = CONFIG.mapper(&target).expect("频道不存在");
        let path = CACHE.file(&id, &url, &channel).await?;
        let receipt = TG_BOT
          .send_text(
            chat_id,
            format!("{}:", html::bold(sender_name.as_str())),
            None,
          )
          .await?;
        DB.put_msg_id_ir_2(&target, &receipt.id, &message.id)?;
        let receipt = if let Some(reply_to) = &message.reply {
          let local_id = DB.get_msg_id_1(&target, reply_to)?;
          TG_BOT.send_image(chat_id, &path, local_id).await?
        } else {
          TG_BOT.send_image(chat_id, &path, None).await?
        };
        DB.put_msg_id_1(&target, &message.id, &receipt.id)?;
      }
      MessageType::Edit { content: _ } => {}
      _ => {}
    }
  }

  Ok(())
}
