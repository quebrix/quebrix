use base64::decode;

use super::{cache::CacheType, get::Get, Cache};

pub trait Strlen {
    fn strlen(&self, cluster: &str, key: &str) -> u32;
}

impl Strlen for Cache {
    fn strlen(&self, cluster: &str, key: &str) -> u32 {
        let result = self.get(cluster, key);
        if result.value.is_some()
            && result
                .value_type
                .is_some_and(|vt| vt.as_i32() == CacheType::Str.as_i32())
        {
            let vec = result.value.unwrap();
            let decoded = decode(vec).unwrap();
            let value = std::str::from_utf8(&decoded).unwrap();
            let len = value.len();
            return len as u32;
        };
        0
    }
}
