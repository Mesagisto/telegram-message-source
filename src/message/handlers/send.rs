use crate::bot::{BotRequester, TG_BOT};
use crate::config::CONFIG;
use crate::ext::db::DbExt;

use arcstr::ArcStr;
use mesagisto_client::data::message::{MessageType, Profile};
use mesagisto_client::data::{message, Packet};
use mesagisto_client::db::DB;
use mesagisto_client::res::RES;
use mesagisto_client::server::SERVER;
use mesagisto_client::EitherExt;
use teloxide::prelude::*;

pub async fn answer_common(msg: Message, _bot: BotRequester) -> anyhow::Result<()> {
  let target = msg.chat.id.0;
  if !CONFIG.bindings.contains_key(&target) {
    return Ok(());
  }
  let address = CONFIG.bindings.get(&target).unwrap().clone();
  let sender = match msg.from() {
    Some(v) => v,
    //fixme
    None => return Ok(()),
  };
  if sender.is_bot {
    return Ok(());
  }
  // let avatar = bot_client().get_user_profile_photos(sender.id).await?;
  let profile = Profile {
    id: sender.id.0.to_be_bytes().into(),
    username: sender.username.clone(),
    nick: Some(sender.full_name()),
  };
  let mut chain = Vec::<MessageType>::new();
  if let Some(text) = msg.text() {
    chain.push(MessageType::Text {
      content: text.to_string(),
    });
  } else if let Some(image) = msg.photo() {
    let photo = image.last().unwrap();
    let file_id: Vec<u8> = photo.file_id.as_bytes().to_vec();
    let uid: Vec<u8> = photo.file_unique_id.as_bytes().to_vec();
    RES.put_image_id(&uid, file_id.clone());
    TG_BOT.file(&uid, &file_id).await?;
    chain.push(MessageType::Image { id: uid, url: None })
  } else if let Some(sticker) = msg.sticker() {
    let file_id: Vec<u8> = sticker.file_id.as_bytes().to_vec();
    let uid: Vec<u8> = sticker.file_unique_id.as_bytes().to_vec();
    RES.put_image_id(&uid, file_id.clone());
    TG_BOT.file(&uid, &file_id).await?;
    chain.push(MessageType::Image { id: uid, url: None });
    if let Some(caption) = msg.caption() {
      chain.push(MessageType::Text {
        content: caption.to_string(),
      });
    }
  } else if let Some(_) = msg.new_chat_members() {
    // TODO
  } else if let Some(_) = msg.left_chat_member() {
    // TODO
  } else if let Some(_) = msg.audio() {
    // TODO
  } else if let Some(_) = msg.animation() {
    // TODO
    // animation is GIF or video
  }

  let reply = match msg.reply_to_message() {
    Some(v) => {
      let local_id = v.id.to_be_bytes().to_vec();
      DB.get_msg_id_2(&target, &local_id).unwrap_or(None)
    }
    None => None,
  };
  DB.put_msg_id_0(&msg.chat.id.0, &msg.id, &msg.id)?;
  let message = message::Message {
    profile,
    id: msg.id.to_be_bytes().to_vec(),
    chain,
    reply,
  };
  let packet = Packet::from(message.tl())?;

  SERVER
    .send(&ArcStr::from(target.to_string()), &address, packet, None)
    .await?;
  Ok(())
}
