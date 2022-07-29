use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Redirect {
    pub key: String,
    pub target: String,
    pub destory_after: NaiveDateTime,
    pub remaining_usage: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRedirct {
    pub target: String,
    // How many seconds should the redirect be valid for?
    pub destory_after: u32,
    // How many times can the redirect be used?
    pub remaining_usage: Option<i64>,
}
