use std::net::TcpListener;

// Why use a separate crate for http client? For decoupling!
use actix_web::rt::spawn;
use reqwest::Client;

use zero2prod::run;

// Spawn the server as a background task, because it should not block the test
fn spawn_app() -> String {
    // Port 0 is a wildcard port that tells the system to find an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to listen");

    let _ = spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[actix_web::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = Client::new();

    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute requeset.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
