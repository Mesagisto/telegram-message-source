use crate::config::CONFIG;

use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "信使Bot支持以下命令")]
pub enum Command {
  #[command(description = "关于本项目")]
  About,
  #[command(description = "显示命令帮助")]
  Help,
  #[command(description = "显示状态")]
  Status,
  #[command(description = "设置当前Group的转发地址", parse_with = "split")]
  SetAddress { address: String },
}
impl Command {
  pub async fn answer(msg: Message, bot: AutoSend<Bot>, cmd: Command) -> anyhow::Result<()> {
    match cmd {
      Command::Help => {
        bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
      }
      Command::SetAddress { address } => {
        let sender_id = msg.from().unwrap().id;
        let chat_id = msg.chat.id;
        let admins = bot.get_chat_administrators(chat_id).await?;
        let mut is_admin = false;
        for admin in admins {
          if admin.user.id == sender_id {
            is_admin = true;
            break;
          }
        }
        if is_admin {
          CONFIG.bindings.insert(chat_id.0, address.into());
          bot
            .send_message(chat_id, "成功设置当前Group的信使地址")
            .await?;
        } else {
          bot
            .send_message(chat_id, "权限不足,拒绝设置信使频道")
            .await?;
        }
      }
      Command::About => {}
      Command::Status => {}
    };
    Ok(())
  }
}
