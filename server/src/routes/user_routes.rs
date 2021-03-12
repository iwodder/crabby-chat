use rocket::request::Form;
use rocket::State;
use crate::user::user_db_service::UserDbService;
use crate::user::{User, NewUserForm, IUser};
use std::sync::Mutex;

use std::error::Error;
use rocket_contrib::json::Json;
use std::collections::HashSet;
use rocket::http::Status;

#[post("/register", data = "<new_user>")]
pub fn register(db: State<Mutex<UserDbService>>, new_user: Form<NewUserForm>) -> Result<Json<User>, Box<dyn Error>> {
    let user = new_user.into_inner();
    let u = User::from_form(user);
    let r = db.lock().unwrap().create_user(Box::new(u))?;
    Ok(Json(r.to_user()))
}

#[post("/<user_id>/favorite", data = "<favorite>")]
pub fn add_favorite(db: State<Mutex<UserDbService>>, user_id: String, favorite: String) -> Result<Json<User>, Status> {
    let service = db.lock().unwrap();
    let user = User {
        user_id: Some(user_id),
        user_name: String::new(),
        favorite_rooms: favorite.split(",").map(|s| String::from(s)).collect::<HashSet<String>>()
    };


    let r = service.update_user(Box::new(user));
    if let Ok(mut found) = r {
        Ok(Json(found.to_user()))
    } else {
        Err(Status::NotFound)
    }
}