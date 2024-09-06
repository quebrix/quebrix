use super::cred_manager::{CredsManager, RoleManagement, User};
use super::cred_manager::{ENCRYPT_KEY, IV_PATTERN};
use crate::crypto::crypto_service::Encryptor;
use crate::known_directories::KNOWN_DIRECTORIES;
use crate::logger::logger_manager::Logger;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
pub trait LoadUserFromFile {
    fn load_users_from_file(&mut self);
}

impl LoadUserFromFile for CredsManager {
    fn load_users_from_file(&mut self) {
        let _ = self.create_admin();
        if self.enable_log == true {
            let message = "default user added";
            let set_log = Logger::log_info(message);
            set_log.write_log_to_file();
        }
        let encryptor = Encryptor::new(ENCRYPT_KEY, IV_PATTERN);

        if self.users.is_empty() {
            self.users = HashMap::new();
        }

        let kn_dir = &KNOWN_DIRECTORIES;
        let open_file_result = {
            OpenOptions::new()
                .read(true)
                .open(PathBuf::from(&kn_dir.creds_directory).join("users.txt"))
        };

        if let Ok(file) = open_file_result {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 3 {
                        let trimmed_input = parts[1].trim_matches(|c| c == '[' || c == ']');
                        let str_numbers = trimmed_input.split(", ");
                        let vec_u8: Vec<u8> = str_numbers
                            .map(|s| s.parse().expect("Invalid byte"))
                            .collect();
                        self.users.insert(
                            parts[0].to_string(),
                            User {
                                username: parts[0].to_string(),
                                password: vec_u8,
                                role: parts[2].parse::<RoleManagement>().unwrap(),
                            },
                        );
                    } else {
                        if self.enable_log == true {
                            let message = &format!("invalid line format : {:?}", line.clone());
                            let set_log = Logger::log_error_data(message);
                            set_log.write_log_to_file();
                        }
                        eprintln!("Invalid line format: {}", line);
                    }
                }
            }
        } else {
            if self.enable_log == true {
                let message = &format!("Failed to open users file path");
                let set_log = Logger::log_info_data(message);
                set_log.write_log_to_file();
            }
            eprintln!("Failed to open users file");
        }
    }
}
