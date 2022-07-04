pub trait LogResultExt<T> {
  fn log_if_error(self, message: &str) -> Option<T>;
}

impl<T> LogResultExt<T> for anyhow::Result<T> {
  #[inline(always)]
  fn log_if_error(self, message: &str) -> Option<T> {
    match self {
      Ok(v) => Some(v),
      Err(e) => {
        tracing::error!(
          "{}, ErrorType {}\n Backtrace {:#?}",
          message,
          e,
          e.backtrace()
        );
        None
      }
    }
  }
}
