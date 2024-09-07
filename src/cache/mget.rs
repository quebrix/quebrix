use std::collections::{HashMap, HashSet};

use base64::decode;
use serde::{Deserialize, Serialize};

use super::{get::Get, Cache};

pub trait MGet {
    fn mget(
        &self,
        cluster_dic: HashMap<String, Vec<String>>,
    ) -> HashMap<String, Vec<MGetKeyValuePair>>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MGetKeyValuePair {
    pub key: String,
    pub value: String,
}

impl MGet for Cache {
    fn mget(
        &self,
        cluster_dic: HashMap<String, Vec<String>>,
    ) -> HashMap<String, Vec<MGetKeyValuePair>> {
        let mut resp_dic = HashMap::<String, Vec<MGetKeyValuePair>>::new();
        for (cluster_name, keys) in cluster_dic.into_iter() {
            let store = self.store.lock().unwrap();
            let cluster = store.get(&cluster_name);
            if cluster.is_none() {
                resp_dic.insert(cluster_name, vec![]);
            } else {
                let mut key_value_vec: Vec<MGetKeyValuePair> = vec![];
                for key in keys {
                    let value = cluster.unwrap().get(&key);
                    if value.is_some() {
                        let vec = &value.unwrap().0;
                        let decoded = decode(vec).unwrap();
                        let value = std::str::from_utf8(&decoded).unwrap();
                        key_value_vec.push(MGetKeyValuePair {
                            key,
                            value: value.to_string(),
                        });
                    }
                }
                resp_dic.insert(cluster_name, key_value_vec);
            }
        }
        resp_dic
    }
}
