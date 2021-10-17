use std::{ops::Deref, sync::Arc};

use arcstr::ArcStr;
use crate::config::CONFIG;
use mesagisto_client::{LateInit, cache::CACHE, res::RES};
use teloxide::{Bot, adaptors::AutoSend, net::Download, prelude::Requester, types::File as TgFile};

#[derive(Singleton, Default)]
pub struct TgBot {
  inner: LateInit<Arc<AutoSend<Bot>>>,
}
impl TgBot {
  pub fn init(&self, bot: Arc<AutoSend<Bot>>) {
    self.inner.init(bot)
  }
  // fixme use this-error
  pub async fn file(&self,uid:&ArcStr,id: &ArcStr) -> anyhow::Result<()>{
    let TgFile{ file_path,.. } = self.get_file(id.as_str()).await.expect("failed to get file");
    let tmp_path = RES.tmp_path(id);
    let mut file = tokio::fs::File::create(&tmp_path).await?;
    // mention: this is stream
    self.inner.download_file(&file_path, &mut file).await?;
    CACHE.put_file(uid, &tmp_path).await?;
    Ok(())
  }
  pub fn get_url_by_path(&self,file_path: String) -> ArcStr{
    format!(
      "https://api.telegram.org/file/bot{token}/{file}",
      token = CONFIG.telegram.token,
      file = file_path
    ).into()
  }
}
impl Deref for TgBot {
  type Target = Arc<AutoSend<Bot>>;
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
