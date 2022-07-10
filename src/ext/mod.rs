use arcstr::{ArcStr, Substr};

pub mod db;
pub mod err;
pub mod res;
pub trait TrimPrefix {
  fn trim_prefix(&self, prefix: &str) -> Option<Substr>;
}
impl TrimPrefix for ArcStr {
  fn trim_prefix(&self, prefix: &str) -> Option<Substr> {
    if self.starts_with(prefix) {
      Some(self.substr(prefix.len()..))
    } else {
      None
    }
  }
}
