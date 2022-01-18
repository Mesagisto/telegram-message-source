use crate::bot::TG_BOT;
use crate::config::CONFIG;
use crate::ext::db::DbExt;
use crate::message::handlers::receive::receive_from_server;
use crate::message::Cx;
use mesagisto_client::data::message::{MessageType, Profile};
use mesagisto_client::data::{message, Packet};
use mesagisto_client::db::DB;
use mesagisto_client::res::RES;
use mesagisto_client::server::SERVER;
use mesagisto_client::EitherExt;
use std::sync::Arc;
use teloxide::prelude::*;

#[cfg(feature = "yinglish")]
pub fn yinglish(text: String) -> String {
  use pyo3::{types::PyModule, Python};

  Python::with_gil(|py| {
    let yinglish = PyModule::import(py, "yinglish").unwrap();
    let res: String = yinglish
      .call_method1("chs2yin", (text,))
      .unwrap()
      .extract()
      .unwrap();
    res
  })
}

pub async fn answer_common(cx: Arc<Cx>) -> anyhow::Result<()> {
  let udp = &cx.update;
  #[cfg(feature = "yinglish")]
  if text.starts_with("!") {
    let mut content = text.clone().to_string();
    content.remove(0);
    let reply_content = yinglish(content);
    cx.reply_to(reply_content).await?;
  }

  let target = cx.chat_id();
  if !CONFIG.target_address_mapper.contains_key(&target) {
    return Ok(());
  }
  let address = CONFIG.target_address_mapper.get(&target).unwrap().clone();
  let sender = match cx.update.from() {
    Some(v) => v,
    //fixme
    None => return Ok(()),
  };
  // let avatar = bot_client().get_user_profile_photos(sender.id).await?;
  let profile = Profile {
    id: sender.id.to_be_bytes().into(),
    username: sender.username.clone(),
    nick: Some(
      sender
        .full_name()
        .replace(|c: char| !c.is_alphanumeric(), ""),
    ),
  };
  let mut chain = Vec::<MessageType>::new();
  if let Some(text) = udp.text() {
    chain.push(MessageType::Text {
      content: text.to_string(),
    });
  } else if let Some(image) = udp.photo() {
    let photo = image.last().unwrap();
    let file_id: Vec<u8> = photo.file_id.as_bytes().to_vec();
    let uid: Vec<u8> = photo.file_unique_id.as_bytes().to_vec();
    RES.put_image_id(&uid, file_id.clone());
    TG_BOT.file(&uid, &file_id).await?;
    chain.push(MessageType::Image { id: uid, url: None })
  } else if let Some(sticker) = udp.sticker() {
    let file_id: Vec<u8> = sticker.file_id.as_bytes().to_vec();
    let uid: Vec<u8> = sticker.file_unique_id.as_bytes().to_vec();
    RES.put_image_id(&uid, file_id.clone());
    TG_BOT.file(&uid, &file_id).await?;
    chain.push(MessageType::Image { id: uid, url: None })
  }

  let reply = match udp.reply_to_message() {
    Some(v) => {
      let local_id = v.id.to_be_bytes().to_vec();
      DB.get_msg_id_2(&target, &local_id).unwrap_or(None)
    }
    None => None,
  };
  DB.put_msg_id_0(&udp.chat_id(), &udp.id, &udp.id)?;
  let message = message::Message {
    profile,
    id: udp.id.to_be_bytes().to_vec(),
    chain,
    reply,
  };
  let packet = Packet::from(message.tl())?;

  SERVER
    .send_and_receive(target.to_be_bytes().to_vec(), address, packet, receive_from_server)
    .await?;
  Ok(())
}
