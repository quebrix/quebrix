use super::cred_manager::{CredsManager, RoleManagement, User};

pub trait IRoleManager {
    fn is_admin(&mut self, user: &User) -> bool;
    fn is_developer(&mut self, user: &User) -> bool;
    //add more role manager functions here
}

impl IRoleManager for CredsManager {
    fn is_admin(&mut self, user: &User) -> bool {
        let user_in_hashmap = self.users.get(&user.username).unwrap();
        if user_in_hashmap.role == RoleManagement::Admin {
            return true;
        } else {
            return false;
        }
    }

    fn is_developer(&mut self, user: &User) -> bool {
        let user_in_hashmap = self.users.get(&user.username).unwrap();
        if user_in_hashmap.role == RoleManagement::Developer {
            return true;
        } else {
            return false;
        }
    }
}
