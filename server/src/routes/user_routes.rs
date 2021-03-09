use rocket::request::Form;
use rocket::State;
use crate::user::user_db_service::UserDbService;
use crate::user::{User, NewUserForm};
use std::sync::Mutex;

use std::error::Error;
use rocket_contrib::json::Json;
use std::collections::HashSet;
use rocket::http::Status;

#[post("/register", data = "<new_user>")]
pub fn register(db: State<Mutex<UserDbService>>, new_user: Form<NewUserForm>) -> Result<Json<User>, Box<dyn Error>> {
    let user = new_user.into_inner();
    let u = User::from_form(user);
    let r = db.lock().unwrap().create_user(u)?;
    Ok(Json(r))
}

#[post("/<user_id>/favorite", data = "<favorite>")]
pub fn add_favorite(db: State<Mutex<UserDbService>>, user_id: String, favorite: String) -> Result<Json<User>, Status> {
    let service = db.lock().unwrap();
    let user = User {
        user_id: Some(user_id),
        user_name: String::new(),
        favorite_rooms: HashSet::new()
    };
    let r = service.retrieve_user(user).unwrap();
    if let Some(mut found) = r {
        found.add_favorites(vec![favorite]);
        service.update_user_favorites(&mut found).unwrap();
        Ok(Json(found))
    } else {
        Err(Status::NotFound)
    }
}