use crate::{Sessions, schema::*};
use crate::models::*;

extern crate bcrypt;
extern crate time;

use diesel::prelude::*;
use bcrypt::{DEFAULT_COST, hash, verify};
use rocket::{http::SameSite, outcome::IntoOutcome, request::Form};
use rocket::request::{self, Request, FromRequest};
use rocket::{State, http::{Cookie, Cookies}};
use rocket::response::{Flash, Redirect};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use time::Duration;
// TODO: String here should be a unchanging reference
pub struct LoginedUser(String);

impl<'a, 'r> FromRequest<'a, 'r> for &'a LoginedUser {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<&'a LoginedUser, !> {
        let sessions = request.guard::<State<Sessions>>().unwrap();

        // This closure will execute at most once per request, regardless of
        // the number of times the `LoginedUser` guard is executed.
        let user_result: &Option<LoginedUser> = request.local_cache(|| {
            // Get the session hashmap
            let session_lock = sessions.lock().unwrap();

            // Get the sessionID from the cookie
            let (username, expires) = request.cookies()
                .get_private("sessionID")
                .and_then(|cookie| cookie.value().parse::<String>().ok())
                .and_then(|session_id| session_lock.get(&session_id))?;

            // Check if it's expired
            let expired = *expires < time::now();

            if !expired {
                request.cookies().get_private("sessionID")?.set_max_age(Duration::hours(1));
                Some(LoginedUser(username.clone()))
            } else {
                None
            }
        });

        user_result.as_ref().or_forward(())
    }
}

// Returns the user's username
#[get("/user")]
pub fn show(user: &LoginedUser) -> String {
    user.0.clone()

    /*
    // This get's all users from the db
    let users: Vec<User> = users::table
        .select(users::all_columns)
        .load::<User>(&crate::users_connection())
        .expect("Failed to connect to db in list()");

    users.iter()
        .map(|u| &u.username)
        .fold(String::new(), |a, b| a + b + "\n")
     */
}

#[derive(FromForm)]
pub struct SignupForm {
    username: String,
    password: String,
}

#[post("/signup", data = "<task>")]
pub fn signup(task: Form<SignupForm>) -> Flash<Redirect> {
    // Parse form 
    let SignupForm { username, password } = task.into_inner();

    // Generate bcrypt hash
    let hashed = hash(password, DEFAULT_COST);

    match hashed {
        Ok(_) => {
            let insert = diesel::insert_into(users::table)
            .values(NewUser {
                username: &username,
                hash: &hashed.unwrap(),            
            })
            .execute(&crate::database_connection());
    
            match insert {
                Ok(_) => Flash::success(Redirect::to("/"), "Success"),
                Err(err_msg) => Flash::error(Redirect::to("/"), format!("error: {}", err_msg))
            }
        }
        Err(err_msg) => Flash::error(Redirect::to("/"), format!("error: {}", err_msg))
    }
}

#[derive(FromForm)]
pub struct SigninForm {
    username: String,
    password: String,
}

#[post("/signin", data = "<task>")]
pub fn signin(task: Form<SigninForm>, mut cookies: Cookies, sessions: State<Sessions>) -> Flash<Redirect> {
    // Parse form 
    let SigninForm { username, password } = task.into_inner();

    // Get hash from db
    let hash = users::table
        .select(users::hash)
        .filter(users::dsl::username.eq(&username))
        .first::<String>(&crate::database_connection());

    // Check we found the user
    match hash {
        Ok(_) => {
            // Validate password
            match verify(password, &hash.unwrap()) {
                Ok(_) => {
                    // Generate a random sessionID
                    let session_id: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(16)
                        .map(char::from)
                        .collect();

                    // Create cookie
                    let cookie = Cookie::build("sessionID",session_id.clone())
                        .http_only(true)
                        .same_site(SameSite::Strict)
                        .max_age(Duration::hours(1))
                        .finish();

                    cookies.add_private(cookie);

                    let session = (username, time::now() + time::Duration::hours(1));
                    sessions.lock().unwrap().insert(session_id.to_string(), session);
                    
                    Flash::success(Redirect::to("/"), "success")
                }
                Err(err_msg) => {
                    println!("BcryptError");
                    Flash::error(Redirect::to("/login"), format!("error: {}", err_msg))
                }
            }
        }
        Err(err_msg) => { 
            println!("Lookup error");
            Flash::error(Redirect::to("/login"), format!("error: {}", err_msg))
        }
    }
}
