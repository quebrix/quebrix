use std::collections::HashMap;

use crate::logger::logger_manager::Logger;

use super::{set::Set, Cache};

pub trait MoveClusterValues {
    fn move_cluster_value(&mut self, src_cluster: &str, desc_cluster: &str) -> bool;
}

impl MoveClusterValues for Cache {
    fn move_cluster_value(&mut self, src_cluster: &str, desc_cluster: &str) -> bool {
        let src_data = {
            let store = self.store.lock().unwrap();
            store.get(src_cluster).cloned()
        };

        if let Some(src_data) = src_data {
            let mut success = true;
            for (key, value) in src_data.iter() {
                let (val, _, _, _) = value;

                if !self.set(
                    desc_cluster.to_string(),
                    key.to_string(),
                    val.to_vec(),
                    None,
                    true,
                ) {
                    Logger::log_error(&format!("Failed to set key {}", key)).write_log_to_file();
                    success = false;
                }
            }

            if success {
                Logger::log_info_data(&format!(
                    "Cluster: {} moved to cluster: {}",
                    src_cluster, desc_cluster
                ))
                .write_log_to_file();

                let mut store = self.store.lock().unwrap();
            }

            success
        } else {
            Logger::log_error("Cluster not found in store for moving cluster values")
                .write_log_to_file();
            false
        }
    }
}
