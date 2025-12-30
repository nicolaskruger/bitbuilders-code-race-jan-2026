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
async fn e2e_full_flow() {
    server_on(|| async {
        let res = reqwest::get("http://localhost:8080/").await.unwrap();
        assert_eq!(res.status(), 200);
    })
    .await;
}
