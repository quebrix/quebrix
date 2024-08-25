
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use actix_web::body::MessageBody;
use serde::Deserialize;

use crate::crypto::crypto_service::Encryptor;

#[derive(Clone, Debug)]
pub struct User {
    username: String,
    password: Vec<u8>, 
    role: RoleManagement,
}

#[derive(Clone, Debug, PartialEq,Deserialize)]
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
}

const ENCRYPT_KEY: &str = "QWERTYUIOPASdfgCBxzjfndeuAQwudhAsd";
const IV_PATTERN: [u8; 16] = [2, 2, 5, 8, 5, 1, 3, 5, 6, 7, 9, 1, 6, 4, 1, 9];

impl CredsManager {
    pub fn new() -> Self {
        let mut creds_manager = CredsManager {
            users: HashMap::new(),
        };
        creds_manager.load_users_from_file();
        creds_manager
    }

    pub fn is_admin(&mut self, user:&User) -> bool {
        let user_in_hashmap = self.users.get(&user.username).unwrap();
        if user_in_hashmap.role == RoleManagement::Admin{
            return true;
        }else{
            return false;
        }
    }

    pub fn get_user(&mut self ,username:&str) ->User{
        self.users.get(username.clone()).unwrap().clone()
    }

    // Add a new user (Admin only)
    pub fn add_user(&mut self, username: String, password: String, role: RoleManagement, current_user: Option<&User>) -> io::Result<()> {
        if current_user.is_some(){
            if !current_user.unwrap().role.can_manage_users() {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied: Admin role required to add users"));
            }
        }
        let encryptor = Encryptor::new(ENCRYPT_KEY, IV_PATTERN);
        let hashed_password = encryptor.encrypt(&password);
        let user = User { username: username.clone(), password: hashed_password, role };
        self.users.insert(username.clone(), user);
        self.write_user_to_file(&username)?;
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
                        println!("Passwords do not match for user: {}", username);
                        return false;
                    }
                }
                None => {
                    println!("Failed to decrypt password for user: {}", username);
                    return false;
                }
            }
        } else {
            println!("User not found: {}:{:?}", username, password);
        }
        false
    }

    fn load_users_from_file(&mut self) {
        let _ = self.add_user("admin".to_string(), "123456".to_string(), RoleManagement::Admin,None);
        let encryptor = Encryptor::new(ENCRYPT_KEY, IV_PATTERN);

        if self.users.is_empty() {
            self.users = HashMap::new();
        }

        let mut main_path = env::current_exe().unwrap();
        let main_file = {
            #[cfg(target_os = "windows")]
            {
                main_path.pop();
                main_path.push("creds/users.txt");
                OpenOptions::new().read(true).open(main_path.clone())
            }
            #[cfg(not(target_os = "windows"))]
            {
                main_path.pop();
                main_path.push(".creds/users.txt");
                OpenOptions::new().read(true).open(main_path)
            }
        };

        if let Ok(file) = main_file {
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
                        self.users.insert(parts[0].to_string(), User {
                            username: parts[0].to_string(),
                            password: vec_u8,
                            role: parts[2].parse::<RoleManagement>().unwrap(),
                        });
                    } else {
                        eprintln!("Invalid line format: {}", line);
                    }
                }
            }
        } else {
            eprintln!("Failed to open users file");
        }
    }

    fn create_admin(&mut self) {
        let encryptor = Encryptor::new(ENCRYPT_KEY, IV_PATTERN);
        let mut path: PathBuf = env::current_exe().unwrap();
        path.push("..");

        #[cfg(target_os = "windows")]
        {
            path.push("creds");

            if !path.exists() {
                fs::create_dir_all(&path).unwrap();
                std::process::Command::new("attrib")
                    .arg("+H")
                    .arg(&path)
                    .output().unwrap();
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            path.push(".creds");

            if !path.exists() {
                fs::create_dir_all(&path);
            }
        }

        path.push("users.txt");

        if !path.exists() {
            fs::File::create(&path).unwrap();
            self.add_user("admin".to_string(), "123456".to_string(), RoleManagement::Admin, None);
        }
    }

    // Write a new user to the file
    fn write_user_to_file(&self, username: &str) -> io::Result<()> {
        let mut path: PathBuf = env::current_exe().unwrap();
        path.push("..");

        #[cfg(target_os = "windows")]
        {
            path.push("creds");

            if !path.exists() {
                fs::create_dir_all(&path)?;
                std::process::Command::new("attrib")
                    .arg("+H")
                    .arg(&path)
                    .output()?;
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            path.push(".creds");

            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
        }

        path.push("users.txt");

        if !path.exists() {
            fs::File::create(&path)?;
        }
        let user = self.users.get(username).unwrap();
        let mut file = OpenOptions::new().append(true).open(path.clone())?;
        writeln!(file, "{}:{:?}:{}", user.username, user.password, user.role.as_str())?;
        Ok(())
    }
}

