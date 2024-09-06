use super::Cache;

pub trait GetAllClusters {
    fn get_all_clusters(&self) -> Vec<String>;
}

impl GetAllClusters for Cache {
    fn get_all_clusters(&self) -> Vec<String> {
        let store = self.store.lock().unwrap();
        store.keys().cloned().collect()
    }
}
