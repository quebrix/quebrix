use crate::{known_directories::KNOWN_DIRECTORIES, logger::logger_manager::Logger, persistent};
use chrono::{prelude::*, Days};
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};

pub fn run_retention_policy(policy_day: i64) {
    let kn_dir = &KNOWN_DIRECTORIES;

    let persistent_dir = &kn_dir.persistent_directory;

    let today = Local::now().date_naive();
    let past_date = today - chrono::Duration::days(policy_day);
    let persistent_file_name = format!("persistent_{}.rus", past_date.format("%d-%m-%Y"));
    let persistent_file_path = PathBuf::from(persistent_dir).join(&persistent_file_name);
    if persistent_file_path.exists() {
        fs::remove_file(&persistent_file_path).expect("can not delete persistent file");
        let message = format!("deleting persis {:?}", &persistent_file_name);
        let log = Logger::log_info_data(&message);
        log.write_log_to_file();
    }
}
