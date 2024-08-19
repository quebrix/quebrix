use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use actix_web::body::MessageBody;

use crate::crypto::crypto_service::Encryptor;

#[derive(Clone, Debug)]
pub struct User {
    username: String,
    password: Vec<u8>, 
}

pub struct CredsManager {
    users: HashMap<String, User>,
}

const ENCRYPT_KEY :&str = "QWERTYUIOPASdfgCBxzjfndeuAQwudhAsd";
const IV_PATTERN :[u8; 16] = [0u8; 16];

impl CredsManager {
    pub fn new() -> Self {
        let mut creds_manager = CredsManager {
            users: HashMap::new(),
        };
        creds_manager.load_users_from_file();
        creds_manager
    }

    // Add a new user
    pub fn add_user(&mut self, username: String, password: String) -> io::Result<()> {
        let encryptor = Encryptor::new(ENCRYPT_KEY,IV_PATTERN);
        let hashed_password = encryptor.encrypt(&password);
        let user = User { username: username.clone(), password: hashed_password };
        self.users.insert(username.clone(), user);

        // Write the user to the file
        self.write_user_to_file(&username)?;

        Ok(())
    }

    // Authenticate a user
    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        let encryptor = Encryptor::new(ENCRYPT_KEY,IV_PATTERN);
        if let Some(user) = self.users.get(username) {
            let encrypted_password = encryptor.decrypt(&user.password).unwrap();
            if (encrypted_password == password.to_string()) {
                return true;
            }
            else{
              return false;
            }
        }
        false
    }

    // Load users from the hidden file
    fn load_users_from_file(&mut self) {
        if let Ok(file) = OpenOptions::new().read(true).open(".creds/users.txt") {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        self.users.insert(parts[0].to_string(), User {
                            username: parts[0].to_string(),
                            password: Vec::from(parts[1].as_bytes().to_vec()),
                        });
                    }
                }
            }
        }
    }

    // Write a new user to the file
    fn write_user_to_file(&self, username: &str) -> io::Result<()> {
        let user = self.users.get(username).unwrap();
        let mut file = OpenOptions::new().append(true).create(true).open(".creds/users.txt")?;
        writeln!(file, "{}:{:?}", user.username, user.password)?;
        Ok(())
    }
}
