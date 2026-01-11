mod auth;

use std::env;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use api::handler::user_handler;
use dotenvy::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn load_pool() -> Pool<Postgres> {
    let db_str = env::var("db_str").unwrap();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_str)
        .await
        .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = load_pool().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(user_handler::create)
            .service(auth::auth)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
