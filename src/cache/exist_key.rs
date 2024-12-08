use super::Cache;

pub trait KeyExists {
    fn exists(&self, cluster: &String, key: &String) -> bool;
}

impl KeyExists for Cache {
    fn exists(&self, cluster: &String, key: &String) -> bool {
        let store = self.store.lock().unwrap();
        let value = store.get(cluster).and_then(|cluster_store| {
            cluster_store
                .get(key)
                .cloned()
                .map(|(value, _, _, _)| value)
        });
        if value.is_some() {
            true
        } else {
            false
        }
    }
}
