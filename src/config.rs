use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub port: u16,
    pub memory_size_limit: usize,
}

impl Settings {
    pub fn new() -> Self {
        let mut current_dir = env::current_exe()
            .expect("Failed to get current directory");
        current_dir.push("..");
        let config_path = current_dir.join("config").join("config.json");
        let file = File::open(&config_path)
            .expect(format!("config.json file not found in {}",config_path.as_os_str().to_str().unwrap()).as_str());
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .expect("Error parsing JSON")
    }
}
