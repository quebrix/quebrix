use super::Cache;

pub trait GetClusterKeys {
    fn get_keys_of_cluster(&self, cluster: &str) -> Option<Vec<String>>;
}

impl GetClusterKeys for Cache {
    fn get_keys_of_cluster(&self, cluster: &str) -> Option<Vec<String>> {
        let store = self.store.lock().unwrap();
        store
            .get(cluster)
            .map(|cluster_store| cluster_store.keys().cloned().collect())
    }
}
