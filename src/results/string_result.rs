use serde::Serialize;

use crate::cache::cache::CacheType;

#[derive(Clone, Serialize)]
pub struct StringResult {
    pub value: Option<String>,
    pub value_type: CacheType,
}

impl StringResult {
    pub fn new(value: Option<String>) -> Self {
        StringResult {
            value,
            value_type: CacheType::Str,
        }
    }
}
