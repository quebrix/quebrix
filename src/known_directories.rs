use crate::Logger;
use std::path::PathBuf;
use std::sync::LazyLock;
pub static KNOWN_DIRECTORIES: LazyLock<KnownDirectories> =
    LazyLock::new(|| KnownDirectories::create_all_known_directories());

pub struct KnownDirectories {
    pub app_root_directory: PathBuf,
    pub log_directory: PathBuf,
    pub data_directory: PathBuf,
    pub persistent_directory: PathBuf,
    pub creds_directory: PathBuf,
}

impl KnownDirectories {
    pub fn create_all_known_directories() -> KnownDirectories {
        // !!!Important!!!: do not use Logger in this scope, using Logger in this scoped will create deadlock

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
                let message = format!(
                    "{:?} directory is not exist create directory ...",
                    &directory
                );
                let set_log = Logger::log_warn_data(&message);
                set_log.write_log_to_file();
                println!(
                    "{:?} directory is not exist create directory ...",
                    &directory
                );

                std::fs::create_dir_all(directory)
                    .expect(format!("Failed to create {:?} directory", &directory).as_str());

                let created_message = format!("{:?} directory has created ", &directory);
                let create_set_log = Logger::log_warn_data(&created_message);
                set_log.write_log_to_file();
                println!("{:?} directory has created", &directory);
            } else {
                let exist_message = format!("{:?} directory already exist", &directory);
                let exist_log = Logger::log_info_data(&exist_message);
                println!("{:?} directory already exist", &directory);
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
