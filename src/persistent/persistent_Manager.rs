use crate::{known_directories::KNOWN_DIRECTORIES, logger::logger_manager::Logger};
use chrono::prelude::*;
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

pub fn write_to_persistent_file(command: &String) -> std::io::Result<()> {
    let kn_dir = &KNOWN_DIRECTORIES.lock().unwrap();

    let now: DateTime<Local> = Local::now();
    let persistent_file_name = format!("persistent_{}.rus", now.format("%d-%m-%Y"));
    let persistent_file_path =
        PathBuf::from(&kn_dir.persistent_directory).join(&persistent_file_name);

    if !persistent_file_path.exists() {
        let set_log = Logger::log_info("persistent file is not exist create file ...");
        set_log.write_log_to_file();
        std::fs::File::create(&persistent_file_path)?;
        let log = Logger::log_info("persistent file created");
        log.write_log_to_file();
    }
    let mut file = OpenOptions::new().append(true).open(persistent_file_path)?;
    let formated_command = format!("{}", command.clone());
    writeln!(file, "{}", formated_command)?;
    let message = format!("command:{} set in persistent", command.clone());
    let command_log = Logger::log_info_data(&message);
    command_log.write_log_to_file();
    Ok(())
}
