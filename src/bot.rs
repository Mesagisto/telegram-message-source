use crate::cover::extract_first_image;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, DispatcherHandlerRx, UpdateWithCx, update_listeners};
use teloxide::error_handlers::OnError;
use teloxide::prelude::*;
use teloxide::{
    net::Download,
    types::{Audio, InputFile},
    utils::command::BotCommand,
};
use teloxide_core::{requests::Requester, types::Message};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn commands_or_message_repl<N,Cmd, CH, MH, FutC, FutM, ErrC, ErrM>(
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
    let cmd_handler = Arc::new(cmd_handler);
    let msg_handler = Arc::new(msg_handler);

    let listener = update_listeners::polling_default(bot.clone()).await;

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
                        match Cmd::parse(&*text_content, clone_bot_name) {
                            Ok(command) => {
                                cmd_handler(cx.to_owned(), command)
                                    .await
                                    .log_on_error()
                                    .await
                            }
                            _ => {
                                msg_handler(cx.to_owned(), String::from(text_content))
                                    .await
                                    .log_on_error()
                                    .await
                            }
                        }
                    };
                    if let Some(audio) = cx.to_owned().update.audio() {
                        audio_handler(cx, audio).await.log_on_error().await;
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

use teloxide_core::types::File as TgFile;

async fn audio_handler(
    cx: Arc<UpdateWithCx<AutoSend<Bot>, Message>>,
    audio: &Audio,
) -> Result<(), anyhow::Error> {
    if let Some(ref cover) = audio.thumb {
        let id = &cover.file_unique_id;
        cx.answer_document(InputFile::FileId(id.to_owned()));
        return Ok(());
    };

    let bot = &cx.requester;
    let file_id = &audio.file_id;

    let TgFile { file_path, .. } = bot.get_file(file_id).await.expect("cannot get file");

    let input_path = &format!("./{}.mp3", &audio.file_unique_id);
    let output_path = &format!("./{}.jpg", &audio.file_unique_id);

    let mut file = tokio::fs::File::create(input_path)
        .await
        .expect("cannot create file");

    cx.answer("downloading audio file").await?;
    bot.download_file(&file_path, &mut file)
        .await
        .expect("cannot download file");

    cx.answer("download finished").await?;

    use std::path::Path;
    extract_first_image(Path::new(input_path), Path::new(output_path))
        .expect("cannot extract cover image");

    cx.answer_photo(InputFile::File(Path::new(output_path).to_path_buf()))
        .await?;
    std::fs::remove_file(input_path).expect("cannot remove temp file");
    std::fs::remove_file(output_path).expect("cannot remove temp file");
    Ok(())
}
