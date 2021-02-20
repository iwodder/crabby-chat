#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_spawn_unchecked)]

use std::net::{IpAddr, SocketAddr};

use rocket_contrib::serve::StaticFiles;

use chat::chat_manager::ChatManager;
use std::sync::Mutex;
use log::info;

mod routes;
mod user;
mod chat;


#[macro_use]
extern crate rocket;

fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    info!("it worked!");

    let mut cm = ChatManager::new();
    cm.run_manager(SocketAddr::new(IpAddr::from([127,0,0,1]), 8080));

    rocket::ignite()
        .manage(Mutex::new(cm))
        .mount("/api", routes![routes::test_routes::hello])
        .mount("/room", routes![chat::chat_routes::create_room, chat::chat_routes::get_rooms])
        .mount("/", StaticFiles::from("static"))
        .launch();
}
