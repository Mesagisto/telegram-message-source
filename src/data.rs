use dashmap::DashMap;
use std::sync::Arc;

#[derive(Educe, Singleton)]
#[educe(Default)]
pub struct Data {
  pub active_endpoint: DashMap<Arc<String>, bool>,
}
