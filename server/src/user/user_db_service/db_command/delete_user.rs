use crate::user::{User, IUser, NullUser};
use crate::user::user_db_service::db_command::DbCommand;
use rusqlite::{Connection, Error, params};

pub struct DeleteUser {
    user: Box<dyn IUser>
}

impl DeleteUser {
    pub fn new(user: Box<dyn IUser>) -> Self {
        DeleteUser {
            user
        }
    }
}

impl DbCommand for DeleteUser {
    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error> {
        let mut delete_stmt = conn.prepare("DELETE FROM users WHERE user_id=?1 AND user_name=?2")?;
        delete_stmt.execute(params![self.user.user_id().unwrap(), self.user.user_name()])?;
        Ok(Box::new(NullUser::new()))
    }
}