use super::{cache::ResultValue, delete::Delete, get::Get, Cache};

pub trait GetDel {
    fn get_del(&self, cluster: &str, key: &str) -> ResultValue;
}

impl GetDel for Cache {
    fn get_del(&self, cluster: &str, key: &str) -> ResultValue {
        let result = self.get(cluster, key);

        if result.value.is_some() {
            self.delete(cluster, key, false);
        }
        result
    }
}
