pub mod get_user;
pub mod create_user;
pub mod update_user;
pub mod delete_user;
pub mod get_favorites;
pub mod update_favorites;

use rusqlite::{Rows, Error, Connection, params};
use crate::user::{IUser, NullUser, User};
use std::collections::HashSet;


pub trait DbCommand {
    fn execute(&mut self, conn: &Connection) -> Result<Box<dyn IUser>, Error>;
}
