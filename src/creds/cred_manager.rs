use crate::known_directories::KNOWN_DIRECTORIES;
use crate::logger::logger_manager::Logger;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::crypto::crypto_service::Encryptor;

#[derive(Clone, Debug)]
pub struct User {
    username: String,
    password: Vec<u8>,
    role: RoleManagement,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum RoleManagement {
    Admin = 0,
    Developer = 1,
}

impl FromStr for RoleManagement {
    type Err = ParseUserRoleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(RoleManagement::Admin),
            "developer" => Ok(RoleManagement::Developer),
            _ => Err(ParseUserRoleError),
        }
    }
}

impl RoleManagement {
    fn as_str(&self) -> &str {
        match self {
            RoleManagement::Admin => "Admin",
            RoleManagement::Developer => "Developer",
        }
    }

    fn can_manage_users(&self) -> bool {
        *self == RoleManagement::Admin
    }

    fn can_manage_cache(&self) -> bool {
        *self == RoleManagement::Admin || *self == RoleManagement::Developer
    }
}

#[derive(Debug)]
pub struct ParseUserRoleError;

impl std::fmt::Display for ParseUserRoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid user role")
    }
}

impl std::error::Error for ParseUserRoleError {}

pub struct CredsManager {
    users: HashMap<String, User>,
    enable_log: bool,
}

const ENCRYPT_KEY: &str = "QWERTYUIOPASdfgCBxzjfndeuAQwudhAsd";
const IV_PATTERN: [u8; 16] = [2, 2, 5, 8, 5, 1, 3, 5, 6, 7, 9, 1, 6, 4, 1, 9];

impl CredsManager {
    pub fn new(enable_log: bool) -> Self {
        let mut creds_manager = CredsManager {
            users: HashMap::new(),
            enable_log: enable_log,
        };
        creds_manager.load_users_from_file();
        if creds_manager.enable_log == true {
            let message = &format!("Users loaded : {:?}", creds_manager.users);
            let set_log = Logger::log_info_data(message);
            set_log.write_log_to_file();
        }
        creds_manager
    }

    pub fn is_admin(&mut self, user: &User) -> bool {
        let user_in_hashmap = self.users.get(&user.username).unwrap();
        if user_in_hashmap.role == RoleManagement::Admin {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_user(&mut self, username: &str) -> User {
        self.users.get(username).unwrap().clone()
    }

    // Add a new user (Admin only)
    pub fn add_user(
        &mut self,
        username: String,
        password: String,
        role: RoleManagement,
        current_user: Option<&User>,
    ) -> io::Result<()> {
        if current_user.is_some() {
            if !current_user.unwrap().role.can_manage_users() {
                if self.enable_log == true {
                    let message = "developer wants to add new profile : Access denied";
                    let set_log = Logger::log_warn(message);
                    set_log.write_log_to_file();
                }
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Permission denied: Admin role required to add users",
                ));
            }
        }
        let encryptor = Encryptor::new(ENCRYPT_KEY, IV_PATTERN);
        let hashed_password = encryptor.encrypt(&password);
        let user = User {
            username: username.clone(),
            password: hashed_password,
            role,
        };
        self.users.insert(username.clone(), user);
        self.write_user_to_file(&username)?;
        if self.enable_log == true {
            let message = &format!("new user added : {:?}", username.clone());
            let set_log = Logger::log_info_data(message);
            set_log.write_log_to_file();
        }
        Ok(())
    }

    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        let encryptor = Encryptor::new(ENCRYPT_KEY, IV_PATTERN);

        if let Some(user) = self.users.get(username) {
            match encryptor.decrypt(&user.password) {
                Some(decrypted_password) => {
                    if decrypted_password == password {
                        return true;
                    } else {
                        if self.enable_log == true {
                            let message =
                                &format!("password not match with user : {:?}", username.clone());
                            let set_log = Logger::log_warn_data(message);
                            set_log.write_log_to_file();
                        }
                        println!("Passwords do not match for user: {}", username);
                        return false;
                    }
                }
                None => {
                    if self.enable_log == true {
                        let message = "Failed to decrypt password for login";
                        let set_log = Logger::log_error(message);
                        set_log.write_log_to_file();
                    }
                    println!("Failed to decrypt password for user: {}", username);
                    return false;
                }
            }
        } else {
            if self.enable_log == true {
                let message = &format!(
                    "Users [{}] with password:[{:?}] not found in login",
                    username, password
                );
                let set_log = Logger::log_info_data(message);
                set_log.write_log_to_file();
            }
            println!("User not found: {}:{:?}", username, password);
        }
        false
    }

    fn load_users_from_file(&mut self) {
        let _ = self.add_user(
            "admin".to_string(),
            "123456".to_string(),
            RoleManagement::Admin,
            None,
        );
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

    fn create_admin(&mut self) {
        let kn_dir = &KNOWN_DIRECTORIES;

        let users_file = PathBuf::from(&kn_dir.creds_directory).join("users.txt");

        if !users_file.exists() {
            fs::File::create(&users_file).unwrap();
            self.add_user(
                "admin".to_string(),
                "123456".to_string(),
                RoleManagement::Admin,
                None,
            )
            .unwrap();
        }
    }

    // Write a new user to the file
    fn write_user_to_file(&self, username: &str) -> io::Result<()> {
        let kn_dir = &KNOWN_DIRECTORIES;

        let users_file = PathBuf::from(&kn_dir.creds_directory).join("users.txt");

        if !users_file.exists() {
            fs::File::create(&users_file)?;
        }

        let user = self.users.get(username).unwrap();
        let mut file = OpenOptions::new().append(true).open(users_file)?;
        writeln!(
            file,
            "{}:{:?}:{}",
            user.username,
            user.password,
            user.role.as_str()
        )?;
        Ok(())
    }
}
