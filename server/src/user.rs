pub mod user_db_service;
mod user_types;
use uuid::Uuid;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::collections::hash_set;
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct NewUserForm {
    user_name: String,
    password: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub(crate) user_id: Option<String>,
    pub(crate) user_name: String,
    pub(crate) favorite_rooms: HashSet<String>
}

impl User {
    pub fn new(user_name: String) -> Self {
        User {
            user_id: None,
            user_name,
            favorite_rooms: HashSet::new()
        }
    }

    pub fn from_form(form: NewUserForm) -> Self {
        User {
            user_id: None,
            user_name: form.user_name,
            favorite_rooms: HashSet::new()
        }
    }

    pub fn user_id(&self) -> Option<&String> {
        self.user_id.as_ref()
    }

    pub fn set_user_id(&mut self, id: String) {
        self.user_id.replace(id);
    }

    pub fn user_name(&self) -> &String {
        &self.user_name
    }

    pub fn set_user_name(&mut self, name: String) {
        self.user_name = name;
    }

    pub fn total_favorites(&self) -> usize {
        self.favorite_rooms.len()
    }

    pub fn add_favorites(&mut self, mut favs: Vec<String>) {
        while let Some(s) = favs.pop() {
            self.favorite_rooms.insert(s);
        }
    }

    pub fn favorites(&self) -> hash_set::Iter<String> {
        self.favorite_rooms.iter()
    }

}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id &&
            self.user_name.to_lowercase() == other.user_name.to_lowercase()
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}


#[cfg(test)]
mod tests {
    use crate::user::User;
    use uuid::Uuid;

    #[test]
    fn new_user_has_no_id() {
        let user = User::new(String::from("jsmith"));
        let id = user.user_id();
        assert!(id.is_none());
    }

    #[test]
    fn can_set_user_id() {
        let mut user = User::new(String::from("dschrute"));
        user.set_user_id(Uuid::new_v4().to_string());
        assert!(user.user_id().is_some());
    }

    #[test]
    fn users_with_same_id_and_name_are_equal() {
        let id = String::from("1234asdf");
        let mut user1 = User::new(String::from("mscott"));
        user1.set_user_id(id.clone());
        let mut user2 = User::new(String::from("mscott"));
        user2.set_user_id(id.clone());
        assert_eq!(user1, user2);
    }

    #[test]
    fn users_without_same_id_and_name_are_not_equal() {
        let id = String::from("1234asdf");
        let mut user1 = User::new(String::from("mscott"));
        user1.set_user_id(id.clone());
        let mut user2 = User::new(String::from("mpalmer"));
        user2.set_user_id(id.clone());
        assert_ne!(user1, user2);
    }

    #[test]
    fn can_add_favorites() {
        let mut user = User::new(String::from("kmalone"));
        assert_eq!(0, user.total_favorites());
        user.add_favorites(
            vec![String::from("chili"), String::from("gambling"),
                 String::from("foot-bath")]);
        assert_eq!(3, user.total_favorites());
    }

    #[test]
    fn favorites_should_not_have_duplicates() {
        let mut user = User::new(String::from("kmalone"));
        assert_eq!(0, user.total_favorites());
        user.add_favorites(
            vec![String::from("chili"), String::from("gambling"),
                 String::from("foot-bath"), String::from("chili")]);
        assert_eq!(3, user.total_favorites());
    }
}