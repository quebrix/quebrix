use std::time::Instant;

use crate::logger::logger_manager::Logger;

use super::{
    cache::{CacheType, ResultValue},
    Cache,
};

pub trait Get {
    fn get(&self, cluster: &str, key: &str) -> ResultValue;
}

impl Get for Cache {
    fn get(&self, cluster: &str, key: &str) -> ResultValue {
        let mut store = self.store.lock().unwrap();

        if let Some(cluster_store) = store.get_mut(cluster) {
            cluster_store.retain(|k, (_, expiration_time, _, _)| {
                if let Some(exp) = expiration_time {
                    exp > &mut Instant::now()
                } else {
                    true
                }
            });
        }
        let value = store.get(cluster).and_then(|cluster_store| {
            cluster_store
                .get(key)
                .cloned()
                .map(|(value, _, _, _)| value)
        });
        let cahe_type = store.get(cluster).and_then(|cluster_store| {
            cluster_store
                .get(key)
                .cloned()
                .map(|(_, _, _, cache_type)| cache_type)
        });
        let mut value_type = None;
        if ((cahe_type.as_ref().is_some())
            && cahe_type.as_ref().unwrap().as_i32() == CacheType::Str.as_i32())
        {
            value_type = Option::Some(CacheType::Str);
        }
        if ((cahe_type.clone().is_some())
            && cahe_type.as_ref().unwrap().as_i32() == CacheType::Int.as_i32())
        {
            value_type = Option::Some(CacheType::Int);
        }
        if self.enable_log == true && value.is_some() {
            let cache_message = format!("value get");
            let get_log = Logger::log_info_data(&cache_message);
            get_log.write_log_to_file();
        }

        return ResultValue {
            value: value,
            value_type: value_type,
        };
    }
}
