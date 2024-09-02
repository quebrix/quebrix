use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::known_directories::KNOWN_DIRECTORIES;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub port: u16,
    pub memory_size_limit: usize,
    pub eviction_strategy: i32,
    pub enable_logger: bool,
    pub persistent: bool,
    pub retention_policy: i64,
}

impl Settings {
    pub fn new() -> Self {
        let current_dir = &KNOWN_DIRECTORIES.lock().unwrap().app_root_directory;

        let config_path = PathBuf::from(current_dir)
            .join("config")
            .join("config.json");
        let file = File::open(&config_path).expect(
            format!(
                "config.json file not found in {}",
                config_path.as_os_str().to_str().unwrap()
            )
            .as_str(),
        );
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Error parsing JSON")
    }
}
