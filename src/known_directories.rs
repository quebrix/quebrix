use std::{path::PathBuf, sync::Mutex};

use lazy_static::lazy_static;

use crate::logger::logger_manager::Logger;

lazy_static! {
    pub static ref KNOWN_DIRECTORIES: Mutex<KnownDirectories> =
        Mutex::new(KnownDirectories::create_all_known_directories());
}

pub struct KnownDirectories {
    pub app_root_directory: PathBuf,
    pub log_directory: PathBuf,
    pub data_directory: PathBuf,
    pub persistent_directory: PathBuf,
    pub creds_directory: PathBuf,
}

impl KnownDirectories {
    pub fn create_all_known_directories() -> KnownDirectories {
        let app_root_directory = KnownDirectories::get_app_root();
        let log_directory = PathBuf::from(&app_root_directory).join("logs");
        let data_directory = PathBuf::from(&app_root_directory).join("data");
        let persistent_directory = PathBuf::from(&data_directory).join("persistent");
        let creds_directory = PathBuf::from(&app_root_directory).join("creds");

        let known_directory_vec = vec![
            &log_directory,
            &data_directory,
            &persistent_directory,
            &creds_directory,
        ];
        known_directory_vec.iter().for_each(|&directory| {
            if !directory.exists() {
                Logger::log_info(
                    format!(
                        "{:?} directory is not exist create directory ...",
                        &directory
                    )
                    .as_str(),
                )
                .write_log_to_file();

                std::fs::create_dir_all(directory)
                    .expect(format!("Failed to create {:?} directory", &directory).as_str());

                Logger::log_info(format!("{:?} directory created", &directory).as_str())
                    .write_log_to_file();
            }
        });

        KnownDirectories {
            app_root_directory,
            data_directory,
            log_directory,
            persistent_directory,
            creds_directory,
        }
    }
    pub fn get_app_root() -> PathBuf {
        let mut file_dir_path: PathBuf = std::env::current_exe().unwrap();
        file_dir_path.pop();
        return file_dir_path;
    }
}
