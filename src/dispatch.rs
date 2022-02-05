use teloxide::prelude2::*;
use tracing::info;

use crate::{command::Command, message::handlers};

pub async fn start(bot: &AutoSend<Bot>) {
  let handler = Update::filter_message()
    .branch(
      dptree::entry()
        .filter_command::<Command>()
        .endpoint(Command::answer),
    )
    .branch(
      dptree::filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
        .endpoint(handlers::send::answer_common),
    );
  info!("Mesagisto-Bot启动成功");
  Dispatcher::builder(bot.clone(), handler)
    .default_handler(|upd| async move {
      log::warn!("Unhandled update: {:?}", upd);
    })
    // If the dispatcher fails for some reason, execute this handler.
    .error_handler(LoggingErrorHandler::with_custom_text(
      "An error has occurred in the dispatcher",
    ))
    .build()
    .setup_ctrlc_handler()
    .dispatch()
    .await;
}
