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

#[derive(Serialize)]
pub struct EntryModel {
    pub name: String,
    pub size: u64,
    pub base: String,
}
#[derive(Serialize)]
pub struct DirectoryModel<'a> {
    pub uri: &'a str,
    pub entries: Vec<EntryModel>
}

impl<'a> DirectoryModel<'a> {
    pub fn new(uri: &'a str, entries: Vec<EntryModel>) -> Self { Self { uri, entries } }
}