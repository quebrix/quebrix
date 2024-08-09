use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::memory_handling;

#[derive(Clone)]
pub struct Cache {
    store: Arc<Mutex<HashMap<String, HashMap<String, Vec<u8>>>>>,
    port: u16,
    memory_handler: Arc<Mutex<memory_handling::memory_handling::MemoryHandler>>,
}

impl Cache {
    pub fn new(port_number: u16, memory_handler:Arc<Mutex<memory_handling::memory_handling::MemoryHandler>>) -> Self {
        Cache {
            store: Arc::new(Mutex::new(HashMap::new())),
            port: port_number,
            memory_handler: memory_handler,
        }
    }

    pub fn set(&self, cluster: String, key: String, value: Vec<u8>) {
        if (!self.memory_handler.lock().unwrap().is_memory_limit_finished()) {
            let mut store = self.store.lock().unwrap();
            let cluster_store = store.entry(cluster.clone()).or_insert_with(HashMap::new);
            let memory_usage = std::mem::size_of_val(&value);
            cluster_store.insert(key, value.clone());
            let mut memory_handler = self.memory_handler.lock().unwrap();
            memory_handler.add_memory(memory_usage);

            println!("Set value to [{}] ", cluster);
        }
        else {
            println!("The memory usage has exceeded the configured limit.update your configuration json file");
            return ;
        }        
    }

    pub fn set_cluster(&self, cluster: String) {
        let mut store = self.store.lock().unwrap();
        store.entry(cluster).or_insert_with(HashMap::new);
    }

    pub fn get(&self, cluster: &str, key: &str) -> Option<Vec<u8>> {
        let store = self.store.lock().unwrap();
        store.get(cluster).and_then(|cluster_store| cluster_store.get(key).cloned())
    }

    pub fn get_keys_of_cluster(&self, cluster: &str) -> Option<Vec<String>> {
        let store = self.store.lock().unwrap();
        store.get(cluster).map(|cluster_store| cluster_store.keys().cloned().collect())
    }

    pub fn delete(&self, cluster: &str, key: &str) {
        let mut store = self.store.lock().unwrap();
        if let Some(cluster_store) = store.get_mut(cluster) {
            if let Some(value) = cluster_store.remove(key) {
                let mut memory_handler = self.memory_handler.lock().unwrap();
                let memory_usage = std::mem::size_of_val(&value);
                memory_handler.delete_memory(memory_usage);
            }
        }
    }

    pub fn clear_cluster(&self, cluster: &str) {
        let mut store = self.store.lock().unwrap();
        if let Some(cluster_store) = store.remove(cluster) {
            let mut memory_handler = self.memory_handler.lock().unwrap();
            let total_size: usize = cluster_store.values().map(|v| std::mem::size_of_val(&v)).sum();
            memory_handler.delete_memory(total_size);
        }
    }

    pub fn clear_all(&self) {
        let mut store = self.store.lock().unwrap();
        let mut memory_handler = self.memory_handler.lock().unwrap();
        let total_size: usize = store.values()
            .flat_map(|cluster_store| cluster_store.values())
            .map(|v| v.len())
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
    
}
