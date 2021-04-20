use dashmap::DashMap;
lazy_static! {
    pub static ref DATA: RuntimeData = RuntimeData::default();
}
pub struct RuntimeData {
    pub active_endpoint:DashMap<i64,bool>
}
impl Default for RuntimeData {
    fn default() -> RuntimeData {
        RuntimeData {
            active_endpoint:DashMap::new()
        }
    }
}
