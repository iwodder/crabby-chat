mod get_user;
mod create_user;
mod update_user;

use rusqlite::{Rows, Error, Connection, params};
use crate::user::{IUser, NullUser, User};
use std::collections::HashSet;


pub trait DbCommand {
    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error>;
}
