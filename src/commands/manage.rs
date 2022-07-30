use arcstr::ArcStr;
use color_eyre::eyre::Result;
use teloxide::{
  prelude::*,
  utils::{command::BotCommands, html},
};

use crate::{bot::BotRequester, config::CONFIG, handlers};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "MesagistoTG management commands")]
pub enum ManageCommand {
  #[command(description = "Disaplay manage commands help")]
  ManageHelp,
  #[command(description = "Add a new NATS Server", parse_with = "split")]
  NewServer { name: String, address: String },
}
impl ManageCommand {
  pub async fn answer(msg: Message, bot: BotRequester, cmd: ManageCommand) -> Result<()> {
    match cmd {
      ManageCommand::ManageHelp => {
        bot
          .send_message(msg.chat.id, ManageCommand::descriptions().to_string())
          .await?;
      }
      ManageCommand::NewServer { name, address } => {}
    }
    Ok(())
  }
}
