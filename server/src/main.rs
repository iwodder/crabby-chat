#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_spawn_unchecked)]

use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;

use rocket_contrib::serve::StaticFiles;

use chat::chat_manager::ChatManager;
use crate::user::user_db_service::UserDbService;
use std::path::Path;

mod routes;
mod chat;
mod user;


#[macro_use]
extern crate rocket;

fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();

    let mut cm = ChatManager::new();
    cm.run(SocketAddr::new(IpAddr::from([127,0,0,1]), 8080));

    let file = std::fs::File::open(Path::new("./test/test_data.sql")).unwrap();;
    let user_db = UserDbService::from_file(file).unwrap();

    rocket::ignite()
        .manage(Mutex::new(cm))
        .manage(Mutex::new(user_db))
        .mount("/room", routes![
        chat::chat_routes::create_room, chat::chat_routes::get_rooms, chat::chat_routes::check_name,
        chat::chat_routes::delete_room])
        .mount("/user", routes![routes::user_routes::register,
        routes::user_routes::add_favorite])
        .mount("/", StaticFiles::from("static"))
        .launch();
}
