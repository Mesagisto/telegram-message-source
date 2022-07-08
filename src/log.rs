use chrono::{Local, Offset, TimeZone};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

pub(crate) fn init() {
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
          time::UtcOffset::from_whole_seconds(
            Local.timestamp(0, 0).offset().fix().local_minus_utc(),
          )
          .unwrap_or(time::UtcOffset::UTC),
          time::macros::format_description!(
            "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
          ),
        )),
    )
    .with(
      tracing_subscriber::filter::Targets::new()
        .with_target("teloxide", Level::INFO)
        .with_target("telegram_message_source", Level::INFO)
        .with_target("mesagisto_client", Level::TRACE)
        .with_default(Level::WARN),
    )
    .init();
}
