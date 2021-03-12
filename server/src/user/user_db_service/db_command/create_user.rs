use rusqlite::{Error, Connection, params};
use crate::user::IUser;
use crate::user::user_db_service::db_command::DbCommand;

pub struct CreateUser {
    user: Box<dyn IUser>
}

impl CreateUser {
    pub fn new(user: Box<dyn IUser>) -> Box<Self> {
        Box::new(CreateUser {
            user
        })
    }
}

impl DbCommand for CreateUser {

    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error> {
        let mut create = conn.prepare("INSERT INTO users (user_id, user_name) VALUES(?1, ?2)")?;
        let id = self.user.user_id().unwrap();
        let user_name = self.user.user_name();
        create.execute(params![id, user_name])?;
        Ok(self.user.to_iuser())
    }
}