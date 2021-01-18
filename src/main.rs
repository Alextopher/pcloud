#![feature(proc_macro_hygiene, decl_macro)]
#![feature(never_type)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

use std::{collections::HashMap, sync::Mutex};

use crate::diesel::Connection;
use diesel::SqliteConnection;
use time::Tm;

pub mod users;
pub mod files;
pub mod models;
pub mod schema;

pub fn database_connection() -> SqliteConnection {
    SqliteConnection::establish(&"cloud.db")
        .expect("Error connecting to cloud.db")
}

// Sessions map cookie's sessionID to username and experiation time
pub type Sessions = Mutex<HashMap<String, (String, Tm)>>;

fn main() {
    let sessions: Sessions = Mutex::new(HashMap::new());

    rocket::ignite()
        .manage(sessions)
        .mount("/", routes![files::public, files::public_root, files::public_upload, files::private_upload])
        .mount("/api/", routes![users::signup, users::signin, users::show])
        .launch();
}
