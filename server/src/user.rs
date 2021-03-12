pub mod user_db_service;
use uuid::Uuid;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::collections::hash_set;
use serde::{Deserialize, Serialize};
use std::collections::hash_set::Iter;

#[derive(FromForm)]
pub struct NewUserForm {
    user_name: String,
    password: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: Option<String>,
    pub user_name: String,
    pub favorite_rooms: HashSet<String>
}

pub trait IUser {

    fn user_id(&self) -> Option<&String>;

    fn set_user_id(&mut self, id: String);

    fn user_name(&self) -> &String;

    fn set_user_name(&mut self, name: String);

    fn total_favorites(&self) -> usize;

    fn add_favorites(&mut self, favs: Vec<String>);

    fn favorites(&self) -> hash_set::Iter<String>;

    fn to_user(&self) -> User;

    fn to_iuser(&self) -> Box<dyn IUser>;
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
}

#[derive(Clone, Debug)]
struct NullUser{
    user_id: Option<String>,
    user_name: String,
    favorite_rooms: HashSet<String>
}

impl NullUser {
    pub fn new() -> Self {
        NullUser {
            user_id: None,
            user_name: String::new(),
            favorite_rooms: HashSet::new()
        }
    }
}

impl IUser for NullUser {
    fn user_id(&self) -> Option<&String> {
        self.user_id.as_ref()
    }

    fn set_user_id(&mut self, id: String) {
        //
    }

    fn user_name(&self) -> &String {
        &self.user_name
    }

    fn set_user_name(&mut self, name: String) {
        //
    }

    fn total_favorites(&self) -> usize {
        0
    }

    fn add_favorites(&mut self, favs: Vec<String>) {
        //
    }

    fn favorites(&self) -> Iter<String> {
        self.favorite_rooms.iter()
    }

    fn to_user(&self) -> User {
        User {
            user_id: None,
            user_name: String::from(""),
            favorite_rooms: HashSet::new()
        }
    }

    fn to_iuser(&self) -> Box<dyn IUser> {
        Box::new(self.clone())
    }
}

impl IUser for User {

    fn user_id(&self) -> Option<&String> {
        self.user_id.as_ref()
    }

    fn set_user_id(&mut self, id: String) {
        self.user_id.replace(id);
    }

    fn user_name(&self) -> &String {
        &self.user_name
    }

    fn set_user_name(&mut self, name: String) {
        self.user_name = name;
    }

    fn total_favorites(&self) -> usize {
        self.favorite_rooms.len()
    }

    fn add_favorites(&mut self, favs: Vec<String>) {
        for fav in favs.iter() {
            self.favorite_rooms.insert(fav.clone());
        }
    }

    fn favorites(&self) -> hash_set::Iter<String> {
        self.favorite_rooms.iter()
    }

    fn to_user(&self) -> User {
        self.clone()
    }

    fn to_iuser(&self) -> Box<dyn IUser> {
        Box::new(self.clone())
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
    use crate::user::{IUser, User};
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