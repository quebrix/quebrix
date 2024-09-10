use super::cred_manager::{CredsManager, User};

pub trait WhowAmI {
    fn who_am_i(&self, user_name: &str) -> User;
}

impl WhowAmI for CredsManager {
    fn who_am_i(&self, user_name: &str) -> User {
        let current_user = self.users.get(user_name).map(|x| x).unwrap();
        User {
            password: vec![],
            role: current_user.clone().role.clone(),
            username: current_user.clone().username,
        }
    }
}
