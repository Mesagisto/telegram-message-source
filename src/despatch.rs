use crate::config::CONFIG;
use crate::webhook::webhook;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use teloxide::dispatching::update_listeners::UpdateListener;
use teloxide::dispatching::{update_listeners, Dispatcher, DispatcherHandlerRx, UpdateWithCx};
use teloxide::error_handlers::{ErrorHandler, OnError};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;
use teloxide_core::types::Message;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn cmd_or_msg_repl<N, Cmd, CH, MH, FutC, FutM>(
  bot: &AutoSend<Bot>,
  bot_name: N,
  cmd_handler: CH,
  msg_handler: MH,
) where
  Cmd: BotCommand + Send + 'static,

  CH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>, Cmd) -> FutC + Send + Sync + 'static,
  FutC: Future<Output = anyhow::Result<()>> + Send + 'static,

  MH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>) -> FutM + Send + Sync + 'static,
  FutM: Future<Output = anyhow::Result<()>> + Send + 'static,

  N: Into<String> + Send + Clone + 'static,
{
  if CONFIG.telegram.webhook.enable {
    let listener = webhook(&bot).await;
    cmd_or_msg_repl_with_listener(&bot, bot_name, cmd_handler, msg_handler, listener).await;
  } else {
    bot
      .delete_webhook()
      .await
      .expect("Failed to delete previous webhook");
    let listener = update_listeners::polling_default(bot.clone()).await;
    cmd_or_msg_repl_with_listener(&bot, bot_name, cmd_handler, msg_handler, listener).await;
  };
}

pub async fn cmd_or_msg_repl_with_listener<N, Cmd, CH, MH, FutC, FutM, L, ListenerE>(
  bot: &AutoSend<Bot>,
  bot_name: N,
  cmd_handler: CH,
  msg_handler: MH,
  listener: L,
) where
  Cmd: BotCommand + Send + 'static,

  CH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>, Cmd) -> FutC + Send + Sync + 'static,
  FutC: Future<Output = anyhow::Result<()>> + Send + 'static,

  MH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>) -> FutM + Send + Sync + 'static,
  FutM: Future<Output = anyhow::Result<()>> + Send + 'static,

  N: Into<String> + Send + Clone + 'static,

  L: UpdateListener<ListenerE> + Send,
  ListenerE: Debug + Send,
{
  let cmd_handler = Arc::new(cmd_handler);
  let msg_handler = Arc::new(msg_handler);

  Dispatcher::new(bot.clone())
    .messages_handler(move |rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
      UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| {
        let msg_handler = Arc::clone(&msg_handler);
        let cmd_handler = Arc::clone(&cmd_handler);
        let clone_bot_name = bot_name.clone();
        let cx = Arc::new(cx);
        async move {
          if let Some(text_content) = cx.clone().update.text() {
            let parse = Cmd::parse(&*text_content, clone_bot_name);
            match parse {
              Ok(cmd) => cmd_handler(cx.to_owned(), cmd).await.log_on_error().await,
              Err(_) => msg_handler(cx).await.log_on_error().await,
            }
          } else {
            msg_handler(cx).await.log_on_error().await
          };
        }
      })
    })
    .setup_ctrlc_handler()
    .dispatch_with_listener(
      listener,
      LoggingErrorHandler::with_custom_text("An error from the update listener"),
    )
    .await
}

struct TracingErrorHandler {
  text: String,
}
impl TracingErrorHandler {
  #[allow(unused)]
  pub fn with_custom_text<T>(text: T) -> Arc<Self>
  where
    T: Into<String>,
  {
    Arc::new(Self { text: text.into() })
  }
}
use futures::future::BoxFuture;
impl ErrorHandler<anyhow::Error> for TracingErrorHandler {
  fn handle_error(self: Arc<Self>, error: anyhow::Error) -> BoxFuture<'static, ()> {
    log::error!("{}:{}, \n Backtrace {}",self.text,error,error.backtrace());
    Box::pin(async {})
  }
}
