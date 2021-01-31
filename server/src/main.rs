#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket_contrib::serve::StaticFiles;
use std::path::Path;

fn main() {

    rocket::ignite()
        .mount("/", StaticFiles::from("static"))
        .launch();
}
