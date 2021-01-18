#![feature(proc_macro_hygiene, decl_macro)]
#![feature(never_type)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

use std::{collections::HashMap, sync::Mutex};

use crate::diesel::Connection;
use diesel::SqliteConnection;
use rocket_contrib::templates::Template;
use time::Tm;

pub mod users;
pub mod public;
pub mod private;
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
        .mount("/", routes![public::public, public::public_file, public::public_directory, private::private, private::private_file, private::private_directory])
        .mount("/api/", routes![users::signup, users::signin, users::show])
        .attach(Template::fairing())
        .launch();
}
