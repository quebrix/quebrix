use super::cred_manager::{CredsManager, User};

pub trait CredUsers {
    fn load_users(&self) -> Vec<User>;
}

impl CredUsers for CredsManager {
    fn load_users(&self) -> Vec<User> {
        self.users
            .values()
            .map(|user| User {
                username: user.username.clone(),
                password: vec![],
                role: user.role.clone(),
            })
            .collect()
    }
}
