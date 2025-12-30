use api::user::User;
use reqwest::Client;
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
        let user = User {
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
