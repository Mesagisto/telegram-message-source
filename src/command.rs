use crate::config::CONFIG;

use teloxide::prelude2::*;
use teloxide::utils::command::BotCommand;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "信使Bot支持以下命令")]
pub enum Command {
  #[command(description = "显示命令帮助")]
  Help,
  // #[command(description = "启用消息转发")]
  // Enable,
  // #[command(description = "禁用消息转发")]
  // Disable,
  #[command(description = "设置当前Group的转发地址", parse_with = "split")]
  SetAddress { address: String },
}
impl Command {
  pub async fn answer(msg: Message, bot: AutoSend<Bot>, cmd: Command) -> anyhow::Result<()> {
    match cmd {
      Command::Help => {
        let chat_id = msg.chat_id();
        bot.send_message(chat_id, Command::descriptions()).await?;
      }
      // Command::Enable => {
      //   cx.answer("Mesagisto信使已启用").await?;
      // }
      // Command::Disable => {
      //   cx.answer("Mesagisto信使已禁用").await?;
      // }
      Command::SetAddress { address } => {
        let sender_id = msg.from().unwrap().id;
        let chat_id = msg.chat_id();
        let admins = bot.get_chat_administrators(chat_id).await?;
        let mut is_admin = false;
        for admin in admins {
          if admin.user.id == sender_id {
            is_admin = true;
            break;
          }
        }
        if is_admin {
          CONFIG.target_address_mapper.insert(chat_id, address.into());
          bot
            .send_message(chat_id, "成功设置当前Group的信使地址")
            .await?;
        } else {
          bot
            .send_message(chat_id, "权限不足,拒绝设置信使频道")
            .await?;
        }
      }
    };
    Ok(())
  }
}
