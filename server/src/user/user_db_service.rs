use rusqlite::{Connection, params, Error};
use crate::user::User;
use uuid::Uuid;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::user::user_db_service::DbServiceError::EmptyFile;

struct UserDbService {
    conn: Connection
}

impl UserDbService {
    fn new() -> Self {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("\
            BEGIN;
            CREATE TABLE users(id TEXT PRIMARY KEY, userName TEXT);
            CREATE TABLE favorites(id INTEGER PRIMARY KEY, user_id TEXT, name TEXT, FOREIGN KEY(user_id) REFERENCES users (id));
            COMMIT;
        \
        ");

        UserDbService {
            conn
        }
    }

    fn from_file(mut file: File) -> Result<Self, Box<dyn StdError>> {
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

    fn create_user(&self, mut new_user: User) -> Result<User, Error> {
        new_user.set_user_id(Uuid::new_v4().to_string());
        let mut create_stmt = self.conn.prepare("INSERT INTO users VALUES(?1, ?2)")?;
        let id = new_user.user_id().unwrap();
        let user_name = new_user.user_name();
        match create_stmt.execute(params![id, user_name]) {
            Ok(_) => Ok(new_user),
            Err(e) => Err(e)
        }
    }

    fn retrieve_user(&self, mut user: User) -> Result<Option<User>, Error> {
        let mut retrieve_stmt = self.conn.prepare("SELECT * FROM users WHERE id=?1 AND userName=?2")?;
        let mut row = retrieve_stmt.query(params![user.user_id().unwrap(), user.user_name()])?;
        if let Some(user_row) = row.next()? {
            Ok(Some(User {
                user_id: user_row.get("id").unwrap(),
                user_name: user_row.get("userName").unwrap(),
                favorite_rooms: HashSet::new()
            }))
        } else {
            Ok(None)
        }
    }

    fn delete_user(&self, mut user: User) -> Result<(), Error> {
        let mut delete_stmt = self.conn.prepare("DELETE FROM users WHERE id=?1 AND userName=?2")?;
        delete_stmt.execute(params![user.user_id().unwrap(), user.user_name()])?;
        Ok(())
    }

    fn update_user(&self, user: &User) -> Result<(), Error> {
        let mut update_stmt = self.conn.prepare("UPDATE users \
        SET userName=?1 WHERE id=?2")?;
        update_stmt.execute(params![user.user_name(), user.user_id().unwrap()]);
        Ok(())
    }

    fn get_user_favorites(&self, mut user: User) -> Result<User, Error> {
        let mut get_favs = self.conn.prepare("SELECT name FROM favorites WHERE user_id=?1")?;
        let mut user_favs:Vec<String> = vec![];
        let mut rows = get_favs.query(params![user.user_id().unwrap()])?;
        while let Some(r) = rows.next()? {
            user_favs.push(r.get(0).unwrap());
        }
        user.add_favorites(user_favs);
        Ok(user)
    }

    fn update_user_favorites(&self, user: &mut User) -> Result<(), Error> {
        let mut update_favs = self.conn.prepare("INSERT INTO favorites (?1, ?2)")?;
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
impl StdError for DbServiceError {
}
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
    use crate::user::User;
    use std::path::Path;
    use std::collections::HashSet;

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
        let retrieved_user = service.retrieve_user(User {
           user_id: Some(String::from("abcd-1234")),
            user_name: String::from("jhalpert"),
            favorite_rooms: HashSet::new()
        });
        assert!(retrieved_user.is_ok());
        assert!(retrieved_user.unwrap().is_some());
    }

    #[test]
    fn creates_new_user() {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(new_user).unwrap();
        assert!(new_user.user_id().is_some());
    }

    #[test]
    fn created_user_is_retrievable() {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(new_user).unwrap();
        let retrieved_user = db_service.retrieve_user(new_user.clone());
        assert_eq!(new_user, retrieved_user.unwrap().unwrap());
    }

    #[test]
    fn can_delete_a_user() {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(new_user).unwrap();
        let result = db_service.delete_user(new_user);
        assert!(result.is_ok());
    }

    #[test]
    fn can_update_a_user() {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(new_user).unwrap();
        new_user.set_user_name(String::from("mscott"));
        db_service.update_user(&new_user);


        let retrieved_user = db_service.retrieve_user(
            new_user.clone()).unwrap().unwrap();
        assert_eq!(new_user, retrieved_user);
    }

    #[test]
    fn new_user_has_zero_favorites() {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(new_user).unwrap();
        new_user = db_service.get_user_favorites(new_user).unwrap();
        assert_eq!(0, new_user.total_favorites());
    }

    #[test]
    fn can_add_user_favorites() {
        let mut db_service = UserDbService::new();
        let mut new_user = User::new(String::from("jhalpert"));
        new_user = db_service.create_user(new_user).unwrap();
        new_user.add_favorites(vec![String::from("chili"), String::from("gambling"),
                 String::from("foot-bath")]);
        db_service.update_user_favorites(&mut new_user);
        new_user = db_service.get_user_favorites(new_user).unwrap();
        assert_eq!(3, new_user.total_favorites());
    }
}