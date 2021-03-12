mod db_command;
use rusqlite::{Connection, params, Error};
use crate::user::{IUser, User};
use uuid::Uuid;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::user::user_db_service::DbServiceError::EmptyFile;
use crate::user::user_db_service::db_command::{delete_user, create_user, get_user, update_user, DbCommand};
use crate::user::user_db_service::db_command::delete_user::DeleteUser;
use crate::user::user_db_service::db_command::update_user::UpdateUser;
use crate::user::user_db_service::db_command::get_user::GetUser;
use crate::user::user_db_service::db_command::create_user::CreateUser;
use crate::user::user_db_service::db_command::get_favorites::GetFavorites;
use crate::user::user_db_service::db_command::update_favorites::UpdateFavorites;

pub struct UserDbService {
    conn: Connection
}

impl UserDbService {
    pub fn new() -> Self {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("\
            BEGIN;
            CREATE TABLE users(id INTEGER PRIMARY KEY, user_id TEXT UNIQUE, user_name TEXT);
            CREATE TABLE favorites(id INTEGER PRIMARY KEY, user_id TEXT, name TEXT, FOREIGN KEY(user_id) REFERENCES users (user_id));
            COMMIT;
        \
        ");

        UserDbService {
            conn
        }
    }

    pub fn from_file(mut file: File) -> Result<Self, Box<dyn StdError>> {
        let mut contents = String::new();
        let read = file.read_to_string(&mut contents);
        if let Ok(_) = read {
            let conn = Connection::open_in_memory()?;
            conn.execute_batch(&contents);
            Ok(UserDbService {
                conn
            })
        } else {
            Err(Box::new(EmptyFile))
        }
    }

    pub fn create_user(&self, mut new_user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        new_user.set_user_id(Uuid::new_v4().to_string());
        CreateUser::new(new_user).execute(&self.conn)
    }

    pub fn retrieve_user(&self, mut user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        let user_id = match user.user_id() {
            Some(id) => id.clone(),
            None => String::new()
        };
        let retrieved_user = GetUser::new(user_id).execute(&self.conn)?;
        GetFavorites::new(retrieved_user).execute(&self.conn)
    }

    pub fn delete_user(&self, mut user: Box<dyn IUser>) -> Result<(), Error> {
        DeleteUser::new(user).execute(&self.conn);
        Ok(())
    }

    pub fn update_user(&self, user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        let updated_user = UpdateUser::new(user.to_iuser()).execute(&self.conn)?;
        UpdateFavorites::new(updated_user).execute(&self.conn)
    }

    pub fn get_user_favorites(&self, mut user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        GetFavorites::new(user).execute(&self.conn)
    }

    pub fn update_user_favorites(&self, user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        UpdateFavorites::new(user).execute(&self.conn)
    }
}

#[derive(Debug)]
pub enum DbServiceError {
    EmptyFile
}
impl StdError for DbServiceError {}
impl Display for DbServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DbServiceError::EmptyFile => write!(f, "File was of zero length, unable to generate")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::user::user_db_service::UserDbService;
    use crate::user::{User, IUser};
    use std::path::Path;
    use std::collections::HashSet;

    fn setup() -> (UserDbService, User) {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(Box::new(new_user)).unwrap().to_user();
        (db_service, new_user)
    }

    #[test]
    fn can_create_db_service() {
        UserDbService::new();
    }

    #[test]
    fn can_initialize_db_with_a_file() {
        let file = std::fs::File::open(Path::new("./test/test_data.sql")).unwrap();
        let result = UserDbService::from_file(file);
        assert!(result.is_ok());
        let service = result.unwrap();
        let retrieved_user = service.retrieve_user(Box::new(User {
           user_id: Some(String::from("abcd-1234")),
            user_name: String::from("jhalpert"),
            favorite_rooms: HashSet::new()
        }));
        assert!(retrieved_user.is_ok());
    }

    #[test]
    fn creates_new_user() {
        let (_, new_user) = setup();
        assert!(new_user.user_id().is_some());
    }

    #[test]
    fn created_user_is_retrievable() {
        let (db_service, new_user) = setup();
        let retrieved_user = db_service.retrieve_user(Box::new(new_user.clone()));
        assert_eq!(new_user, retrieved_user.unwrap().to_user());
    }

    #[test]
    fn can_delete_a_user() {
        let (db_service, new_user) = setup();
        let result = db_service.delete_user(Box::new(new_user));
        assert!(result.is_ok());
    }

    #[test]
    fn can_update_a_user() {
        let (db_service, mut new_user) = setup();
        new_user.set_user_name(String::from("mscott"));
        let updated_user = db_service.update_user(Box::new(new_user.clone())).unwrap();


        let retrieved_user = db_service.retrieve_user(
            Box::new(new_user.clone())).unwrap();
        assert_eq!(updated_user.to_user(), retrieved_user.to_user());
    }

    #[test]
    fn new_user_has_zero_favorites() {
        let (db_service, mut new_user) = setup();
        new_user = db_service.retrieve_user(Box::new(new_user)).unwrap().to_user();
        assert_eq!(0, new_user.total_favorites());
    }

    #[test]
    fn can_add_user_favorites() {
        let (db_service, mut new_user) = setup();
        new_user.add_favorites(vec![String::from("chili"), String::from("gambling"),
                 String::from("foot-bath")]);
        db_service.update_user(new_user.to_iuser()).unwrap();
        let found_user = db_service.retrieve_user(new_user.to_iuser()).unwrap();
        assert_eq!(3, found_user.total_favorites());
    }

    #[test]
    fn non_existent_user_returns_null_user() {
        let user = Box::new(User::new(String::from("jhalpert")));
        let db_service = UserDbService::new();
        let found = db_service.retrieve_user(user);
        assert!(found.is_ok());
        let found_user = found.unwrap();
        assert!(found_user.user_id().is_none())
    }
}