use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::requests::ResponseResult;

pub async fn answer_msg(
    cx: Arc<UpdateWithCx<AutoSend<Bot>, Message>>,
    content: &String,
) -> ResponseResult<bool> {
    if content.eq("echo") {
        cx.answer("echo").await?;
        return respond(true);
    }
    if content.starts_with("!") {
        let reply_content = format!("You just wrote: {}", content);
        cx.reply_to(reply_content).await?;
    }
    respond(false)
}




