use crate::{types::*, AppState};
use actix_web::{get, http, post, web, HttpResponse};
use rand::{distributions::Alphanumeric, prelude::*};
use sqlx::Sqlite;

#[get("/r")]
async fn list_redirects(data: web::Data<AppState>) -> HttpResponse {
    let redirects = sqlx::query_as!(Redirect, "SELECT * FROM redirects")
        .fetch_all(&data.pool)
        .await
        .unwrap();

    HttpResponse::Ok().json(redirects)
}

#[post("/r")]
async fn create_redirect(data: web::Data<AppState>, req: web::Json<CreateRedirct>) -> HttpResponse {
    if url::Url::parse(&req.target).is_err() {
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).body("Invalid URL");
    }

    if let Some(remaining_usage) = req.remaining_usage {
        if remaining_usage <= 0 {
            return HttpResponse::build(http::StatusCode::BAD_REQUEST)
                .body("Invalid remaining usage");
        }
    }

    // Keep generating random strings until we find one that's not in the database.
    loop {
        // Start a transaction.
        let mut tx = data.pool.begin().await.unwrap();

        let key = rand_key();

        // Check if the key is already in the database.
        let stmt = sqlx::query_as!(Redirect, "SELECT * FROM redirects WHERE key = ?", key)
            .fetch_one(&mut tx)
            .await;

        match stmt {
            Ok(_) => {
                continue;
            }
            Err(_) => {
                // Build the redirect.
                let r = Redirect {
                    key: key.clone(),
                    target: req.target.clone(),
                    destory_after: chrono::Utc::now().naive_utc()
                        + chrono::Duration::seconds(req.destory_after as i64),
                    remaining_usage: req.remaining_usage,
                };

                // Insert the redirect into the database.
                let _insert = sqlx::query!("INSERT INTO redirects (key, target, destory_after, remaining_usage) VALUES (?, ?, ?, ?)", r.key, r.target, r.destory_after, r.remaining_usage)
                    .execute(&mut tx)
                    .await;

                // Commit the transaction.
                match tx.commit().await {
                    Ok(_) => {
                        return HttpResponse::Ok().json(r);
                    }
                    Err(_) => {
                        return HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .body("Failed to create redirect");
                    }
                }
            }
        }
    }
}

#[get("/r/{key}")]
async fn get_redirect(data: web::Data<AppState>, key: web::Path<String>) -> HttpResponse {
    let key = key.into_inner();

    match get_and_update_redirect(&data.pool, key).await {
        Ok(redirect) => HttpResponse::TemporaryRedirect()
            .append_header((http::header::LOCATION, redirect.target))
            .finish(),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::build(http::StatusCode::NOT_FOUND).body("Not found")
        }
        Err(_) => {
            // error 500
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal server error")
        }
    }
}

// Gets a record from the database, validates that it is still valid, and then returns it.
// If the record is not valid then it is deleted from the database.
async fn get_and_update_redirect(
    pool: &sqlx::Pool<Sqlite>,
    key: String,
) -> Result<Redirect, sqlx::Error> {
    let mut err = sqlx::Error::RowNotFound;

    for _ in 0..10 {
        // Begin a transaction.
        let mut tx = pool.begin().await.unwrap();

        // Update the remaining usage.
        let _ = sqlx::query!(
            "UPDATE redirects SET remaining_usage = remaining_usage - 1 WHERE key = ?",
            key
        )
        .execute(&mut tx)
        .await;

        let now = chrono::Utc::now().naive_utc();

        // Check if the record should be deleted.
        let _ = sqlx::query!(
            "DELETE FROM redirects WHERE key = ? AND (remaining_usage < 0 OR destory_after < ?)",
            key,
            now
        )
        .execute(&mut tx)
        .await;

        // Get the record from the database.
        let redirect = sqlx::query_as!(Redirect, "SELECT * FROM redirects WHERE key = ?", key)
            .fetch_one(&mut tx)
            .await?;

        // Commit the transaction.
        match tx.commit().await {
            Ok(_) => return Ok(redirect),
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(e);
                }
                _ => err = e,
            },
        }
    }

    Err(err)
}

// Gets a random unused key to be used as a new url
// Keys are [a-z0-9]{5}
fn rand_key() -> String {
    let mut rng = rand::thread_rng();

    (0..5)
        .map(|_| rng.sample(Alphanumeric) as char)
        .map(|c| {
            if !c.is_ascii_digit() {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect()
}
