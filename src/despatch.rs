use crate::webhook::webhook;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use teloxide::dispatching::update_listeners::UpdateListener;
use teloxide::dispatching::{Dispatcher, DispatcherHandlerRx, UpdateWithCx, update_listeners};
use teloxide::error_handlers::OnError;
use teloxide::prelude::*;
use teloxide::{
    utils::command::BotCommand,
};
use teloxide_core::types::Message;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::config::CONFIG;

pub async fn cmd_or_msg_repl<N,Cmd, CH, MH, FutC, FutM, ErrC, ErrM>(
    bot: AutoSend<Bot>,
    bot_name: N,
    cmd_handler: CH,
    msg_handler: MH,
) where
    Cmd: BotCommand + Send + 'static,

    CH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>, Cmd) -> FutC + Send + Sync + 'static,
    FutC: Future<Output = Result<(), ErrC>> + Send + 'static,
    Result<(), ErrC>: OnError<ErrC>,
    ErrC: Debug + Send,

    MH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>, String) -> FutM + Send + Sync + 'static,
    FutM: Future<Output = Result<(), ErrM>> + Send + 'static,
    Result<(), ErrM>: OnError<ErrM>,
    ErrM: Debug + Send,

    N: Into<String> + Send +Clone + 'static
{

   if CONFIG.telegram.webhook.enable {
        let listener =  webhook(&bot).await;
        cmd_or_msg_repl_with_listener(bot, bot_name, cmd_handler, msg_handler, listener).await;
    } else {
        let listener =  update_listeners::polling_default(bot.clone()).await;
        cmd_or_msg_repl_with_listener(bot, bot_name, cmd_handler, msg_handler, listener).await;
    };
}

pub async fn cmd_or_msg_repl_with_listener<'a,N,Cmd, CH, MH, FutC, FutM, ErrC, ErrM,L,ListenerE>(
    bot: AutoSend<Bot>,
    bot_name: N,
    cmd_handler: CH,
    msg_handler: MH,
    listener: L,
) where
    Cmd: BotCommand + Send + 'static,

    CH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>, Cmd) -> FutC + Send + Sync + 'static,
    FutC: Future<Output = Result<(), ErrC>> + Send + 'static,
    Result<(), ErrC>: OnError<ErrC>,
    ErrC: Debug + Send,

    MH: Fn(Arc<UpdateWithCx<AutoSend<Bot>, Message>>, String) -> FutM + Send + Sync + 'static,
    FutM: Future<Output = Result<(), ErrM>> + Send + 'static,
    Result<(), ErrM>: OnError<ErrM>,
    ErrM: Debug + Send,

    N: Into<String> + Send +Clone + 'static,

    L: UpdateListener<ListenerE> + Send + 'a,
    ListenerE: Debug + Send + 'a
{
    let cmd_handler = Arc::new(cmd_handler);
    let msg_handler = Arc::new(msg_handler);

    Dispatcher::new(bot.clone())
    .messages_handler(
        move |rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| {
                let msg_handler = Arc::clone(&msg_handler);
                let cmd_handler = Arc::clone(&cmd_handler);
                let clone_bot_name = bot_name.clone();
                let cx = Arc::new(cx);
                async move {
                    if let Some(text_content) = cx.clone().update.text() {
                        if text_content.starts_with("/") {
                            let cmd = Cmd::parse(&*text_content, clone_bot_name).unwrap();
                            cmd_handler(cx.to_owned(), cmd).await.log_on_error().await
                        } else {
                            msg_handler(cx.to_owned(), String::from(text_content)).await.log_on_error().await
                        }
                    };
                }
            })
        },
    )
    .setup_ctrlc_handler()
    .dispatch_with_listener(
        listener,
        LoggingErrorHandler::with_custom_text("An error from the update listener"),
    )
    .await
}
