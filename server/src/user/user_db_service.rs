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
use crate::user::user_db_service::db_command::{CreateUser, DbCommand, GetUser, UpdateUser};

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
        GetUser::new(user_id).execute(&self.conn)
    }

    pub fn delete_user(&self, mut user: Box<dyn IUser>) -> Result<(), Error> {
        let mut delete_stmt = self.conn.prepare("DELETE FROM users WHERE user_id=?1 AND user_name=?2")?;
        delete_stmt.execute(params![user.user_id().unwrap(), user.user_name()])?;
        Ok(())
    }

    pub fn update_user(&self, user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        UpdateUser::new(user).execute(&self.conn)
    }

    pub fn get_user_favorites(&self, mut user: Box<dyn IUser>) -> Result<Box<dyn IUser>, Error> {
        let mut get_favs = self.conn.prepare("SELECT name FROM favorites WHERE user_id=?1")?;
        let mut user_favs:Vec<String> = vec![];
        let mut rows = get_favs.query(params![user.user_id().unwrap()])?;
        while let Some(r) = rows.next()? {
            user_favs.push(r.get(0).unwrap());
        }
        user.add_favorites(user_favs);
        Ok(user)
    }

    pub fn update_user_favorites(&self, user: &mut dyn IUser) -> Result<(), Error> {
        let mut update_favs = self.conn.prepare("INSERT INTO favorites (user_id, name) VALUES (?1, ?2)")?;
        let id = user.user_id().unwrap();
        for f in user.favorites() {
            update_favs.execute(params![id, f]);
        }
        Ok(())
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
        new_user = db_service.get_user_favorites(Box::new(new_user)).unwrap().to_user();
        assert_eq!(0, new_user.total_favorites());
    }

    #[test]
    fn can_add_user_favorites() {
        let (db_service, mut new_user) = setup();
        new_user.add_favorites(vec![String::from("chili"), String::from("gambling"),
                 String::from("foot-bath")]);
        db_service.update_user_favorites(&mut new_user);
        new_user = db_service.get_user_favorites(Box::new(new_user)).unwrap().to_user();
        assert_eq!(3, new_user.total_favorites());
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