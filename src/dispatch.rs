use teloxide::prelude::*;
use tracing::info;

use crate::{bot::BotRequester, command::Command, message::handlers};

pub async fn start(bot: &BotRequester) {
  let message_handler = Update::filter_message()
    .branch(
      dptree::entry()
        .filter_command::<Command>()
        .endpoint(Command::answer),
    )
    .branch(
      dptree::filter(|msg: Message| {
        msg.chat.is_group() || msg.chat.is_supergroup() || msg.chat.is_channel()
      })
      .endpoint(handlers::send::answer_common),
    );
  let edit_message_handler = Update::filter_edited_message().branch(
    dptree::filter(|msg: Message| {
      msg.chat.is_group() || msg.chat.is_supergroup() || msg.chat.is_channel()
    })
    .endpoint(handlers::send::answer_common),
  );
  let handler = dptree::entry()
    .branch(message_handler)
    .branch(edit_message_handler);

  info!("Mesagisto信使启动成功");
  Dispatcher::builder(bot.clone(), handler)
    .error_handler(LoggingErrorHandler::with_custom_text(
      "调度器中发生了一个错误",
    ))
    .build()
    .setup_ctrlc_handler()
    .dispatch()
    .await;
}
