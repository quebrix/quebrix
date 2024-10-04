use super::Cache;

pub trait KeysCount {
    fn keys_count(&self, cluster: &str) -> i64;
}

impl KeysCount for Cache {
    fn keys_count(&self, cluster: &str) -> i64 {
        let store = self.store.lock().unwrap();
        let key_count = store.get(cluster).map(|k| k.keys().len() as i64);
        key_count.unwrap_or(-1)
    }
}
