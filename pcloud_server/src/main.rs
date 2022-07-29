mod auth;
mod redirect;
pub mod types;

use actix_web::{middleware, web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sqlx::SqlitePool;

struct AppState {
    pool: SqlitePool,
}

async fn db_setup(pool: &SqlitePool) {
    let queries = vec!["CREATE TABLE IF NOT EXISTS redirects (key TEXT NOT NULL PRIMARY KEY, target TEXT NOT NULL, destory_after DATETIME NOT NULL, remaining_usage INTEGER)"];
    for query in queries {
        sqlx::query(query).execute(pool).await.unwrap();
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create TLS keys for testing.
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    // Open the database.
    let pool = SqlitePool::connect("sqlite://db.sqlite3?mode=rwc")
        .await
        .unwrap();
    db_setup(&pool).await;
    let data = web::Data::new(AppState { pool });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(data.clone())
            .service(redirect::list_redirects)
            .service(redirect::create_redirect)
            .service(redirect::get_redirect)
    })
    .bind_openssl("127.0.0.1:8080", builder)?
    .run()
    .await
}
