pub struct GetUser {
    user_id: String
}

impl GetUser {
    pub fn new(user_id: String) -> Self {
        GetUser {
            user_id
        }
    }
}

impl DbCommand for GetUser {

    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error> {
        let mut retrieve_stmt = conn.prepare("SELECT * FROM users WHERE user_id=?1")?;
        let mut row = retrieve_stmt.query(params![self.user_id])?;
        if let Some(user_row) = row.next()? {
            let mut user = User::new(user_row.get("user_name").unwrap());
            user.set_user_id(user_row.get("user_id").unwrap());
            Ok(Box::new(user))
        } else {
            Ok(Box::new(NullUser::new()))
        }
    }
}