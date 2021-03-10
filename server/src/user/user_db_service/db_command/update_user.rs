pub struct UpdateUser {
    user: Box<dyn IUser>
}

impl UpdateUser {
    pub fn new(user: Box<dyn IUser>) -> Self {
        UpdateUser {
            user
        }
    }
}

impl DbCommand for UpdateUser {
    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error> {
        let mut update_stmt = conn.prepare("UPDATE users SET user_name=?1 WHERE user_id=?2")?;
        update_stmt.execute(params![self.user.user_name(), self.user.user_id().unwrap()]);
        Ok(self.user.to_iuser())
    }
}