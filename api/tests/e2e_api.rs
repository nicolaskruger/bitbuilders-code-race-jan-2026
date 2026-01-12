use actix_web::http::StatusCode;
use api::entity::auth_entity::AuthMe;
use api::entity::user_entity::UserRegister;
use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::future::Future;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

fn start_server() -> Child {
    Command::new("cargo")
        .args(["run", "--bin", "api"])
        .spawn()
        .expect("failed to start server")
}

async fn server_on<F, Fut>(callback: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut server = start_server();
    sleep(Duration::from_secs(2)); // aguarda subir

    callback().await;

    server.kill().unwrap();
}

#[tokio::test]
#[ignore = "e2e"]
async fn hello_world() {
    server_on(|| async {
        let res = reqwest::get("http://localhost:8080/").await.unwrap();
        assert_eq!(res.status(), 200);
    })
    .await;
}

#[tokio::test]
#[ignore = "e2e"]
async fn no_user_bad_request() {
    server_on(|| async {
        let client = Client::new();
        let res = client
            .post("http://localhost:8080/user")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
    })
    .await;
}

#[tokio::test]
#[ignore = "e2e"]
async fn no_user_empty_user_bad_request() {
    server_on(|| async {
        let client = Client::new();
        let res = client
            .post("http://localhost:8080/user")
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
    })
    .await;
}

#[tokio::test]
#[ignore = "e2e"]
async fn full_body_ok() {
    server_on(|| async {
        let user = UserRegister {
            name: String::from("name"),
            password: String::from("password"),
        };
        let client = Client::new();
        let res = client
            .post("http://localhost:8080/user")
            .json(&user)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
    })
    .await;
}

async fn load_pool() -> Pool<Postgres> {
    dotenv().ok();
    let db_str = env::var("db_str").unwrap();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_str)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "e2e"]
async fn e2e_register_user() {
    let pool = load_pool().await;

    server_on(|| async {
        let user = UserRegister {
            name: String::from("name"),
            password: String::from("password"),
        };
        let client = Client::new();
        let res = client
            .post("http://localhost:8080/user")
            .json(&user)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
    })
    .await;

    sqlx::query("DELETE FROM users WHERE name = $1")
        .bind("name")
        .execute(&pool)
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

#[tokio::test]
#[ignore = "e2e"]
async fn e2e_fetch_token() {
    let pool = load_pool().await;

    sqlx::query("DELETE FROM users WHERE name = $1")
        .bind("name")
        .execute(&pool)
        .await
        .unwrap();

    server_on(|| async {
        let user = UserRegister {
            name: String::from("name"),
            password: String::from("password"),
        };
        let client = Client::new();

        let res = client
            .post("http://localhost:8080/user")
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), 201);

        let res = client
            .post("http://localhost:8080/auth")
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), 200);

        let auth_token = res.json::<AuthToken>().await.unwrap();

        assert!(auth_token.token.len() > 10);
    })
    .await;

    sqlx::query("DELETE FROM users WHERE name = $1")
        .bind("name")
        .execute(&pool)
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    name: String,
}

#[tokio::test]
#[ignore = "e2e"]
async fn e2e_me() {
    let pool = load_pool().await;

    sqlx::query("DELETE FROM users WHERE name = $1")
        .bind("name")
        .execute(&pool)
        .await
        .unwrap();

    server_on(|| async {
        let user = UserRegister {
            name: String::from("name"),
            password: String::from("password"),
        };
        let client = Client::new();

        let res = client
            .post("http://localhost:8080/user")
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), 201);

        let res = client
            .post("http://localhost:8080/auth")
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), 200);

        let auth_token = res.json::<AuthToken>().await.unwrap();

        assert!(auth_token.token.len() > 10);

        let res = client
            .get("http://localhost:8080/auth/me")
            .bearer_auth(auth_token.token)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status().as_u16(), StatusCode::OK.as_u16());

        let user = res.json::<AuthMe>().await.unwrap();

        assert_eq!(user.name, String::from("name"));
    })
    .await;

    sqlx::query("DELETE FROM users WHERE name = $1")
        .bind("name")
        .execute(&pool)
        .await
        .unwrap();
}
