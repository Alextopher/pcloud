/* Import macros and others */
use crate::schema::*;

/* For beeing able to serialize */
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct User {
    pub username: String,
    pub hash: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name="users"]
pub struct NewUser<'x> {
    pub username: &'x str,
    pub hash: &'x str,
}