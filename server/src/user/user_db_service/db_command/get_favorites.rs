use crate::user::user_db_service::db_command::DbCommand;
use rusqlite::{Connection, Error, params};
use crate::user::{IUser, User, NullUser};

pub struct GetFavorites {
    user: Box<dyn IUser>
}

impl GetFavorites {
    pub fn new(user: Box<dyn IUser>) -> Self {
        GetFavorites {
            user
        }
    }
}

impl DbCommand for GetFavorites {

    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error> {
        let mut get_favs = conn.prepare("SELECT name FROM favorites WHERE user_id=?1")?;
        let mut user_favs:Vec<String> = vec![];
        if let Some(id) = self.user.user_id() {
            let mut rows = get_favs.query(params![id])?;
            while let Some(r) = rows.next()? {
                user_favs.push(r.get(0).unwrap());
            }
            self.user.add_favorites(user_favs);
        }
        Ok(self.user.to_iuser())
    }
}