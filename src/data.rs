use std::sync::Arc;
use once_cell::sync::Lazy;
use dashmap::DashMap;

pub static DATA: Lazy<RuntimeData> = Lazy::new(|| {
    RuntimeData::default()
});
#[derive(Educe)]
#[educe(Default)]
pub struct RuntimeData {
    pub active_endpoint:DashMap<Arc<String>,bool>
}
