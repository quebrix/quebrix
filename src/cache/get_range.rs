use base64::decode;

use crate::results::string_result::StringResult;

use super::{cache::CacheType, get::Get, Cache};

pub trait GetRange {
    fn get_range(&self, cluster: &str, key: &str, start: u32, end: u32) -> StringResult;
}

impl GetRange for Cache {
    fn get_range(&self, cluster: &str, key: &str, start: u32, end: u32) -> StringResult {
        let result = self.get(cluster, key);
        if result.value.is_some() {
            if result.value_type.unwrap().as_i32() == CacheType::Str.as_i32() {
                let vec = result.value.unwrap();
                let decoded = decode(vec).unwrap();
                let value = std::str::from_utf8(&decoded).unwrap();

                let mut indices = value.char_indices().map(|(i, _)| i);
                let start_i = indices.nth(start as usize).unwrap();
                let end_i = indices.nth((end - start) as usize).unwrap_or(value.len());
                let value = String::from(&value[start_i..end_i]);

                return StringResult::new(Some(value));
            }
        }
        StringResult::new(None)
    }
}
