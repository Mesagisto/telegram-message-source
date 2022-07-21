use chrono::{Local, Offset, TimeZone};
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;

pub(crate) fn init() {
  let mut filter = tracing_subscriber::filter::Targets::new()
    .with_target("teloxide", Level::INFO)
    .with_target("telegram_message_source", Level::INFO)
    .with_target("mesagisto_client", Level::TRACE)
    .with_default(Level::WARN);

  if cfg!(feature = "tokio-console") {
    filter = filter
      .with_target("tokio", Level::TRACE)
      .with_target("runtime", Level::TRACE);
  }

  let registry = tracing_subscriber::registry()
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
    .with(ErrorLayer::default())
    .with(filter);

  #[cfg(feature = "tokio-console")]
  registry.with(console_subscriber::spawn()).init();
  #[cfg(not(feature = "tokio-console"))]
  registry.init();
}
