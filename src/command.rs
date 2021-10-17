use crate::config::CONFIG;
use std::sync::Arc;
use arcstr::ArcStr;
use teloxide::dispatching::UpdateWithCx;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "信使Bot支持以下命令")]
pub enum Command {
  #[command(description = "显示命令帮助")]
  Help,
  #[command(description = "启用消息转发")]
  Enable,
  #[command(description = "禁用消息转发")]
  Disable,
  #[command(description = "设置当前Group的转发地址", parse_with = "split")]
  SetAddress { address: String },
}

pub async fn answer(
  cx: Arc<UpdateWithCx<AutoSend<Bot>, Message>>,
  command: Command,
) -> anyhow::Result<()> {
  match command {
    Command::Help => {
      cx.answer(Command::descriptions()).await?;
    }
    Command::Enable => {
      cx.answer("Mesagisto信使已启用").await?;
    }
    Command::Disable => {
      cx.answer("Mesagisto信使已禁用").await?;
    }
    Command::SetAddress { address } => {
      // fixme check permission
      CONFIG
        .target_address_mapper
        .insert(cx.chat_id(), ArcStr::from(address));
      cx.answer("成功设置当前Group的信使地址").await?;
    }
  };
  Ok(())
}
