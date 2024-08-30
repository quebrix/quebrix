use std::{env, fs::{self, File}};
use chrono::{prelude::*, Days};
use crate::logger::logger_manager::Logger;

pub fn run_retention_policy(policy_day:i64){
    let mut path = env::current_exe().unwrap();
    path.pop();
    let today = Local::now().date_naive();
    let past_date = today - chrono::Duration::days(policy_day);
    let persistent_file_name = format!("persistent_{}.rus",past_date.format("%d-%m-%Y"));
    path.push(persistent_file_name.clone());
    if path.exists(){
        fs::remove_file(path.clone()).expect("can not delete persistent file");
        let message = format!("deleting persis {:?}",persistent_file_name);
        let log = Logger::log_info_data(&message);
        log.write_log_to_file();
    }
}