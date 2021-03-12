use crate::user::user_db_service::db_command::DbCommand;
use rusqlite::{Connection, Error, params};
use crate::user::{IUser, User, NullUser};

pub struct UpdateFavorites {
    user: Box<dyn IUser>
}

impl UpdateFavorites {
    pub fn new(user: Box<dyn IUser>) -> Self {
        UpdateFavorites {
            user
        }
    }
}

impl DbCommand for UpdateFavorites {

    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error> {
        let mut update_favs = conn.prepare("INSERT INTO favorites (user_id, name) VALUES (?1, ?2)")?;
        let id = self.user.user_id().unwrap();
        for f in self.user.favorites() {
            update_favs.execute(params![id, f]);
        }
        Ok(self.user.to_iuser())
    }
}