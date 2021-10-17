use std::sync::Arc;
use teloxide::prelude::*;

use super::handlers::send::answer_common;
pub async fn answer_msg(cx: Arc<UpdateWithCx<AutoSend<Bot>, Message>>) -> anyhow::Result<()> {
  answer_common(cx).await
}
