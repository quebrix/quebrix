use std::{fs::{File, OpenOptions}, io::Write, path::PathBuf,path::Path};
use chrono::prelude::*;
pub enum LogType{
    Information,
    Error,
    Warning
}

pub struct Logger<'a> {
    message: &'a str,
    log_type: LogType,
}

impl<'a> Logger<'a> {
    pub fn log_info(message: &'a str) -> Self {
        Logger {
            message,
            log_type: LogType::Information,
        }
    }

    pub fn log_error(message: &'a str) -> Self {
        Logger {
            message,
            log_type: LogType::Error,
        }
    }

    pub fn log_warn(message: &'a str) -> Self {
        Logger {
            message,
            log_type: LogType::Warning,
        }
    }

    pub fn write_log_to_file(&self) {
        let mut file_dir_path:PathBuf = std::env::current_dir().unwrap().join("Log"); 
        let now: DateTime<Local> = Local::now();
        if !file_dir_path.exists() {
            std::fs::create_dir_all(file_dir_path.clone());
            file_dir_path.push("logger.txt");
            if !file_dir_path.exists(){
                File::create(file_dir_path.clone()).expect("Failed to create logger file");
            }
        }
        file_dir_path.push("logger.txt");
        let mut file = OpenOptions::new()
            .append(true)
            .open(file_dir_path.clone())
            .expect("Failed to open logger file");
        let formatted_message = match self.log_type {
            LogType::Information => format!("[INFO] at:{:?} =>: {}\n",now, self.message),
            LogType::Error => format!("[ERROR] at:{:?} =>:  {}\n",now, self.message),
            LogType::Warning => format!("[WARNING] at:{:?} =>: {}\n",now, self.message),
        };
        file.write_all(formatted_message.as_bytes())
            .expect("Failed to write to logger file");
    }
}