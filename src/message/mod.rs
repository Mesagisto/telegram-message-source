use teloxide::{adaptors::AutoSend, prelude::UpdateWithCx, types::Message, Bot};

pub mod handler;
pub mod handlers;

type Cx = UpdateWithCx<AutoSend<Bot>, Message>;
