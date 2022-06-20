use crate::ext::db::DbExt;
use crate::CONFIG;
use crate::TG_BOT;
use arcstr::ArcStr;
use mesagisto_client::LateInit;
use mesagisto_client::{
  cache::CACHE,
  data::{message::Message, message::MessageType, Packet},
  db::DB,
  server::SERVER,
};
use teloxide::types::ChatId;
use teloxide::types::InputFile;
use teloxide::utils::markdown;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

static CHANNEL: LateInit<UnboundedSender<(i64, ArcStr)>> = LateInit::new();

pub fn recover() -> anyhow::Result<()> {
  let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(i64, ArcStr)>();
  tokio::spawn(async move {
    for element in rx.recv().await {
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
  let target:i64 = target.parse()?;
  log::trace!("接收到来自目标{}的消息", target);
  let packet = Packet::from_cbor(&message.data)?;
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
    log::trace!("正在处理消息链中的元素");
    match single {
      MessageType::Text { content } => {
        let content = format!(
          "*{}*:\n{}",
          markdown::escape(&sender_name.as_str()),
          markdown::escape(&content.as_str())
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
            format!("*{}*:", markdown::escape(&sender_name.as_str())),
            None,
          )
          .await?;
        DB.put_msg_id_ir_2(&target, &receipt.id, &message.id)?;
        let receipt = if let Some(reply_to) = &message.reply {
          let local_id = DB.get_msg_id_1(&target, reply_to)?;
          TG_BOT
            .send_image(chat_id, InputFile::file(path), local_id)
            .await?
        } else {
          TG_BOT
            .send_image(chat_id, InputFile::file(path), None)
            .await?
        };
        DB.put_msg_id_1(&target, &message.id, &receipt.id)?;
      }
    }
  }

  Ok(())
}
