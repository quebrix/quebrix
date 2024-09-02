use std::{env, fs::{File, OpenOptions}, io::{BufReader, Write}, path::PathBuf};
use std::fmt;
use chrono::prelude::*;
use crate::{logger::logger_manager::Logger, main};

pub fn write_to_persistent_file(command: &String) -> std::io::Result<()> {
    let mut path: PathBuf = env::current_exe().unwrap();
    path.pop();

    #[cfg(target_os = "windows")]
    {
        path.push("data/persistent");

        if !path.exists() {
            let set_log = Logger::log_info("data/persistent directory is not exist create directory ...");
            set_log.write_log_to_file();
            std::fs::create_dir_all(&path)?;
            std::process::Command::new("attrib")
                .arg("+H")
                .arg(&path)
                .output()?;
            let log = Logger::log_info("data/persistent directory created");
            log.write_log_to_file();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.push(".data/persistent");

        if !path.exists() {
            let set_log = Logger::log_info("data/persistent directory is not exist create directory ...");
            set_log.write_log_to_file();
            std::fs::create_dir_all(&path)?;
            let log = Logger::log_info("data/persistent directory created");
            log.write_log_to_file();
        }
    }
    let now: DateTime<Local> = Local::now();
    let persistent_file_name = format!("persistent_{}.rus",now.format("%d-%m-%Y"));
    path.push(persistent_file_name);

    if !path.exists() {
        let set_log = Logger::log_info("persistent file is not exist create file ...");
        set_log.write_log_to_file();
        std::fs::File::create(&path)?;
        let log = Logger::log_info("persistent file created");
        log.write_log_to_file();
    }
    let mut file = OpenOptions::new().append(true).open(path.clone())?;
    let formated_command = format!("{}", command.clone());
    writeln!(file, "{}", formated_command)?;
    let message = format!("command:{} set in persistent",command.clone());
    let command_log = Logger::log_info_data(&message);
    command_log.write_log_to_file();
    Ok(())
}





