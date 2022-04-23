use std::ops::Deref;

use crate::{config::CONFIG, command::Command};
use arcstr::ArcStr;
use mesagisto_client::{cache::CACHE, net::NET, res::RES, LateInit};
use teloxide::{adaptors::AutoSend, prelude::Requester, types::File as TgFile, Bot, utils::command::BotCommands};

#[derive(Singleton, Default)]
pub struct TgBot {
  inner: LateInit<AutoSend<Bot>>,
}
impl TgBot {
  pub async fn init(&self, bot: AutoSend<Bot>) -> anyhow::Result<()> {
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
}
impl Deref for TgBot {
  type Target = AutoSend<Bot>;
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
