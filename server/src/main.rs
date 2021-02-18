#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_spawn_unchecked)]

use std::net::{IpAddr, Ipv4Addr, TcpListener, SocketAddr};

use rocket_contrib::serve::StaticFiles;

use chat::chat_manager::ChatManager;

mod routes;
mod user;
mod chat;


#[macro_use]
extern crate rocket;

fn main() {
    let mut cm = ChatManager::new();
    cm.run_manager(SocketAddr::new(IpAddr::from([127,0,0,1]), 8000));

    // rocket::ignite()
    //     .mount("/api", routes![routes::test_routes::hello])
    //     .mount("/", StaticFiles::from("static"))
    //     .launch();
}
