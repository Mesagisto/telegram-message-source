use std::ops::Deref;

use arcstr::ArcStr;
use lateinit::LateInit;
use mesagisto_client::{cache::CACHE, net::NET, res::RES};
use teloxide::{
  adaptors::{AutoSend, DefaultParseMode},
  payloads::{SendAnimationSetters, SendMessageSetters, SendPhotoSetters},
  prelude::Requester,
  types::{File as TgFile, InputFile, Message},
  utils::command::BotCommands,
  Bot,
};
use teloxide_core::types::ChatId;
use tracing::warn;

use crate::{command::Command, config::CONFIG, message::handlers};
pub type BotRequester = AutoSend<DefaultParseMode<Bot>>;

#[derive(Singleton, Default)]
pub struct TgBot {
  inner: LateInit<BotRequester>,
}
impl TgBot {
  pub async fn init(&self, bot: BotRequester) -> anyhow::Result<()> {
    bot.set_my_commands(Command::bot_commands()).await?;
    self.inner.init(bot);
    Ok(())
  }

  // fixme use this-error
  pub async fn file(&self, uid: &Vec<u8>, id: &Vec<u8>) -> anyhow::Result<()> {
    let id_str: ArcStr = base64_url::encode(id).into();
    let TgFile { file_path, .. } = self
      .get_file(String::from_utf8_lossy(id))
      .await
      .expect("获取文件失败");
    let tmp_path = RES.tmp_path(&id_str);
    let url = self.get_url_by_path(file_path);
    NET.download(&url, &tmp_path).await?;
    CACHE.put_file(uid, &tmp_path).await?;
    Ok(())
  }

  pub fn get_url_by_path(&self, file_path: String) -> ArcStr {
    format!(
      "https://api.telegram.org/file/bot{token}/{file}",
      token = CONFIG.telegram.token,
      file = file_path
    )
    .into()
  }

  pub async fn send_text<T>(
    &self,
    chat_id: ChatId,
    text: T,
    reply: Option<i32>,
  ) -> anyhow::Result<teloxide::types::Message>
  where
    T: Into<String> + Clone,
  {
    let send = self.inner.send_message(chat_id, text.clone());
    let send = if let Some(reply) = reply {
      send.reply_to_message_id(reply)
    } else {
      send
    };
    match send.await {
      Ok(ok) => Ok(ok),
      Err(e) => match e {
        teloxide::RequestError::MigrateToChatId(new_id) => {
          let target = chat_id.0;
          warn!("Chat migrated from {} to {}", target, new_id);
          if let Some(address) = CONFIG.migrate_chat(&target, &new_id) {
            handlers::receive::del(target)?;
            handlers::receive::add(new_id, &address)?;
            let send = TG_BOT.send_message(ChatId(new_id), text.clone());
            let receipt = if let Some(reply) = reply {
              send.reply_to_message_id(reply).await?
            } else {
              send.await?
            };
            Ok(receipt)
          } else {
            Err(e.into())
          }
        }
        _ => Err(e.into()),
      },
    }
  }

  pub async fn send_image(
    &self,
    chat_id: ChatId,
    image_path: &std::path::Path,
    reply: Option<i32>,
  ) -> anyhow::Result<teloxide::types::Message> {
    let photo = InputFile::file(image_path);
    let kind = infer::get_from_path(image_path)
      .expect("file read failed when refering file type")
      .expect("Unkown file type");
    let is_gif = "gif" == kind.extension();

    let result = if is_gif {
      let photo = photo.clone().file_name(format!(
        "{:?}.gif",
        image_path.file_name().expect("Wrong filename")
      ));
      let send = self.inner.send_animation(chat_id, photo.clone());
      if let Some(reply) = reply {
        send.reply_to_message_id(reply).await
      } else {
        send.await
      }
    } else {
      let send = self.inner.send_photo(chat_id, photo.clone());
      if let Some(reply) = reply {
        send.reply_to_message_id(reply).await
      } else {
        send.await
      }
    };

    match result {
      Ok(ok) => Ok(ok),
      Err(e) => match e {
        teloxide::RequestError::MigrateToChatId(new_id) => {
          let target = chat_id.0;
          warn!("Chat migrated from {} to {}", target, new_id);
          if let Some(address) = CONFIG.migrate_chat(&target, &new_id) {
            handlers::receive::del(target)?;
            handlers::receive::add(new_id, &address)?;
            let receipt: Message = if is_gif {
              let send = self.inner.send_animation(chat_id, photo.clone());
              if let Some(reply) = reply {
                send.reply_to_message_id(reply).await?
              } else {
                send.await?
              }
            } else {
              let send = self.inner.send_photo(chat_id, photo.clone());
              if let Some(reply) = reply {
                send.reply_to_message_id(reply).await?
              } else {
                send.await?
              }
            };
            Ok(receipt)
          } else {
            Err(e.into())
          }
        }
        _ => Err(e.into()),
      },
    }
  }
}

impl Deref for TgBot {
  type Target = BotRequester;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
