use super::{cache::CacheType, Cache};

pub trait TypeOfKey {
    fn get_type(&self, cluster: &String, key: &String) -> Option<CacheType>;
}

impl TypeOfKey for Cache {
    fn get_type(&self, cluster: &String, key: &String) -> Option<CacheType> {
        let store = self.store.lock().unwrap();
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
        return value_type;
    }
}
