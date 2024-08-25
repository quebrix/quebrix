use std::collections::HashMap;
use std::ptr::null;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;
use crate::logger::logger_manager::Logger;
use crate::creds::cred_manager::CredsManager;

use crate::memory_handling;

#[derive(Clone)]
pub struct Cache {
    evict_type:i32,
    store: Arc<Mutex<HashMap<String, HashMap<String, (Vec<u8>, Option<Instant>, Option<Duration>)>>>>,
    port: u16,
    memory_handler: Arc<Mutex<memory_handling::memory_handling::MemoryHandler>>,
    enable_log:bool,
    creds_manager: Arc<Mutex<CredsManager>>
}

impl Cache {
    pub fn new(port_number: u16, memory_handler: Arc<Mutex<memory_handling::memory_handling::MemoryHandler>>,evict_type:i32,enable_logs:bool,creds: Arc<Mutex<CredsManager>>) -> Self {
        Cache {
            store: Arc::new(Mutex::new(HashMap::new())),
            port: port_number,
            memory_handler,
            evict_type:evict_type,
            enable_log:enable_logs,
            creds_manager:creds
        }
    }

    pub fn set(&self, cluster: String, key: String, value: Vec<u8>, ttl: Option<Duration>,user_name:&str,password:&str) -> bool{
        if !self.creds_manager.lock().unwrap().authenticate(user_name, password){
            println!("access denied login first");
            return false;
        }
        else{
            let memory_usage = std::mem::size_of_val(&value);

            // Check if the memory limit is reached
            {
                let mut memory_handler = self.memory_handler.lock().unwrap();
                if memory_handler.is_memory_limit_finished() {
                    println!("Memory limit exceeded. Evicting entries...");
                    self.evict_entries();
                    if self.enable_log == true {
                        let  memory_handler_log = Logger::log_warn("Memory limit exceeded. Evicting entries");
                        memory_handler_log.write_log_to_file();
                    }
                }
            }

            if !self.memory_handler.lock().unwrap().is_memory_limit_finished() {
                let mut store = self.store.lock().unwrap();
                let cluster_store = store.entry(cluster.clone()).or_insert_with(HashMap::new);
                let expiration_time = ttl.map(|duration| Instant::now() + duration);
                cluster_store.insert(key, (value.clone(), expiration_time, ttl));
                let mut memory_handler = self.memory_handler.lock().unwrap();
                memory_handler.add_memory(memory_usage);

                println!("Set value in cluster [{}]", &cluster);
                if self.enable_log == true {
                    let  set_log = Logger::log_info("Set value in cluster");
                    set_log.write_log_to_file();
                }
                return true;
            } else {
                println!("Failed to set value: Memory usage has exceeded the configured limit. Update your configuration JSON file.");
                if self.enable_log == true {
                    let  error_set_log = Logger::log_info("Failed to set value: Memory usage has exceeded the configured limit. Update your configuration JSON file.");
                    error_set_log.write_log_to_file();
                }
                return false;
            }
        }
    }

    pub fn set_cluster(&self, cluster: String) {
        let mut store = self.store.lock().unwrap();
        store.entry(cluster).or_insert_with(HashMap::new);
        if self.enable_log == true {
            let  set_cluster_log = Logger::log_info("cluster set ");
            set_cluster_log.write_log_to_file();
        }
    }

    pub fn get(&self, cluster: &str, key: &str) -> Option<Vec<u8>> {
        
            let mut store = self.store.lock().unwrap();
        
            if let Some(cluster_store) = store.get_mut(cluster) {
                cluster_store.retain(|k, (_, expiration_time, _)| {
                    if let Some(exp) = expiration_time {
                        exp > &mut Instant::now()
                    } else {
                        true
                    }
                });
            }
            let result = store.get(cluster).and_then(|cluster_store| cluster_store.get(key).cloned().map(|(value, _, _)| value));
            if self.enable_log == true && result.is_some() {
                let  get_log = Logger::log_info("value get ");
                get_log.write_log_to_file();
            }
            return result;
    }

    pub fn get_keys_of_cluster(&self, cluster: &str) -> Option<Vec<String>> {
        let store = self.store.lock().unwrap();
        store.get(cluster).map(|cluster_store| cluster_store.keys().cloned().collect())
    }

    pub fn delete(&self, cluster: &str, key: &str) {
        let mut store = self.store.lock().unwrap();
        if let Some(cluster_store) = store.get_mut(cluster) {
            if let Some((value, _, _)) = cluster_store.remove(key) {
                let mut memory_handler = self.memory_handler.lock().unwrap();
                let memory_usage = std::mem::size_of_val(&value);
                memory_handler.delete_memory(memory_usage);
                if self.enable_log == true {
                    let  delete_log = Logger::log_info("value deleted ");
                    delete_log.write_log_to_file();
                }
            }
        }
    }

    pub fn clear_cluster(&self, cluster: &str) {
        let mut store = self.store.lock().unwrap();
        if let Some(cluster_store) = store.remove(cluster) {
            let mut memory_handler = self.memory_handler.lock().unwrap();
            let total_size: usize = cluster_store.values().map(|(v, _, _)| std::mem::size_of_val(v)).sum();
            memory_handler.delete_memory(total_size);
            if self.enable_log == true {
                let  clear_cluster_log = Logger::log_info("cluster cleared ");
                clear_cluster_log.write_log_to_file();
            }
        }
    }

    pub fn clear_all(&self) {
        let mut store = self.store.lock().unwrap();
        let mut memory_handler = self.memory_handler.lock().unwrap();
        let total_size: usize = store.values()
            .flat_map(|cluster_store| cluster_store.values())
            .map(|(v, _, _)| v.len())
            .sum();
        store.clear();
        memory_handler.delete_memory(total_size);
    }

    pub fn get_all_clusters(&self) -> Vec<String> {
        let store = self.store.lock().unwrap();
        store.keys().cloned().collect()
    }

    pub fn configure_default_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn get_default_port(&self) -> u16 {
        self.port
    }

    pub fn clear_expired(&self) {
        let mut store = self.store.lock().unwrap();
    
        for cluster_store in store.values_mut() {
            cluster_store.retain(|_, (_, expiration_time, _)| {
                if let Some(exp) = expiration_time {
                    exp > &mut Instant::now()
                } else {
                    true
                }
            });
        }
    }

    // Eviction strategies
    fn evict_entries(&self) {
        let mut store = self.store.lock().unwrap();
        let mut memory_handler = self.memory_handler.lock().unwrap();
        let eviction_strategy = EvictionStrategy::from_i32(self.evict_type);
        match eviction_strategy.unwrap() {
            EvictionStrategy::VolatileLru => self.evict_volatile_lru(&mut store, &mut memory_handler),
            EvictionStrategy::AllKeysLru => self.evict_allkeys_lru(&mut store, &mut memory_handler),
            EvictionStrategy::AllKeysRandom => self.evict_allkeys_random(&mut store, &mut memory_handler),
            //EvictionStrategy::VolatileRandom => self.evict_volatile_random(&mut store, &mut memory_handler),
            EvictionStrategy::VolatileTtl => self.evict_volatile_ttl(&mut store, &mut memory_handler),
        }
    }

    fn evict_volatile_lru(&self, store: &mut HashMap<String, HashMap<String, (Vec<u8>, Option<Instant>, Option<Duration>)>>, memory_handler: &mut memory_handling::memory_handling::MemoryHandler) {
        let mut lru_key: Option<(String, String)> = None;

        // Find the least recently used key among those with an expiration set
        for (cluster_key, cluster_store) in store.iter() {
            for (key, (_, expiration_time, _)) in cluster_store {
                if let Some(exp) = expiration_time {
                    if exp > &Instant::now() {
                        // Update LRU key logic here
                        if lru_key.is_none() {
                            lru_key = Some((cluster_key.clone(), key.clone()));
                        }
                    }
                }
            }
        }

        // Evict the LRU key
        if let Some((cluster_key, key_to_evict)) = lru_key {
            if let Some(cluster_store) = store.get_mut(&cluster_key) {
                if let Some((value, _, _)) = cluster_store.remove(&key_to_evict) {
                    let memory_usage = std::mem::size_of_val(&value);
                    memory_handler.delete_memory(memory_usage);
                    println!("Evicted [{}] from cluster [{}] using volatile LRU strategy", key_to_evict, cluster_key);
                }
            }
        }
    }

    fn evict_volatile_ttl(&self, store: &mut HashMap<String, HashMap<String, (Vec<u8>, Option<Instant>, Option<Duration>)>>, memory_handler: &mut memory_handling::memory_handling::MemoryHandler) {
        let mut shortest_ttl_key: Option<(String, String, Instant)> = None;

        // Find the key with the shortest TTL among those with an expiration set
        for (cluster_key, cluster_store) in store.iter() {
            for (key, (_, expiration_time, _)) in cluster_store {
                if let Some(exp) = expiration_time {
                    if exp > &Instant::now() {
                        if shortest_ttl_key.is_none() {
                            shortest_ttl_key = Some((cluster_key.clone(), key.clone(), *exp));
                        }
                    }
                }
            }
        }

        // Evict the key with the shortest TTL
        if let Some((cluster_key, key_to_evict, _)) = shortest_ttl_key {
            if let Some(cluster_store) = store.get_mut(&cluster_key) {
                if let Some((value, _, _)) = cluster_store.remove(&key_to_evict) {
                    let memory_usage = std::mem::size_of_val(&value);
                    memory_handler.delete_memory(memory_usage);
                    println!("Evicted [{}] from cluster [{}] using volatile TTL strategy", key_to_evict, cluster_key);
                }
            }
        }
    }


    fn evict_allkeys_lru(&self, store: &mut HashMap<String, HashMap<String, (Vec<u8>, Option<Instant>, Option<Duration>)>>, memory_handler: &mut memory_handling::memory_handling::MemoryHandler) {
        let mut lru_key: Option<(String, String)> = None;

        // Find the least recently used key regardless of expiration
        for (cluster_key, cluster_store) in store.iter() {
            for (key, (_, expiration_time, _)) in cluster_store {
                // Update LRU key logic here
                if lru_key.is_none() {
                    lru_key = Some((cluster_key.clone(), key.clone()));
                }
            }
        }

        // Evict the LRU key
        if let Some((cluster_key, key_to_evict)) = lru_key {
            if let Some(cluster_store) = store.get_mut(&cluster_key) {
                if let Some((value, _, _)) = cluster_store.remove(&key_to_evict) {
                    let memory_usage = std::mem::size_of_val(&value);
                    memory_handler.delete_memory(memory_usage);
                    println!("Evicted [{}] from cluster [{}] using allkeys LRU strategy", key_to_evict, cluster_key);
                }
            }
        }
    }

    fn evict_allkeys_random(&self, store: &mut HashMap<String, HashMap<String, (Vec<u8>, Option<Instant>, Option<Duration>)>>, memory_handler: &mut memory_handling::memory_handling::MemoryHandler) {
        let keys: Vec<(String, String)> = store.iter()
            .flat_map(|(cluster_key, cluster_store)| {
                cluster_store.iter()
                    .map(|(key, (value, _, _))| (cluster_key.clone(), key.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        if let Some((cluster_key, key_to_evict)) = keys.choose(&mut rand::thread_rng()) {
            if let Some(cluster_store) = store.get_mut(cluster_key) {
                if let Some((value, _, _)) = cluster_store.remove(key_to_evict) {
                    let memory_usage = std::mem::size_of_val(&value);
                    memory_handler.delete_memory(memory_usage);
                    println!("Evicted [{}] from cluster [{}] using allkeys random strategy", key_to_evict, cluster_key);
                }
            }
        }
    }
}

//strategy
enum EvictionStrategy {
    VolatileLru,
    VolatileTtl,
    AllKeysLru,
    AllKeysRandom,
}
impl EvictionStrategy {
    pub fn from_i32(value: i32) -> Option<EvictionStrategy> {
        match value {
            0 => Some(EvictionStrategy::VolatileLru),
            1 => Some(EvictionStrategy::VolatileTtl),
            2 => Some(EvictionStrategy::AllKeysLru),
            3 => Some(EvictionStrategy::AllKeysRandom),
            _ => {
                println!("invalid EvictionStrategy passing in json");
                return None;
            }
        }
    }
}