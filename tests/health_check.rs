// Why use a separate crate for http client? For decoupling!
use actix_web::rt::spawn;
use reqwest::Client;
use zero2prod::run;

// Spawn the server as a background task, because it should not block the test
fn spawn_app() {
    let server = run().expect("Failed to bind address");

    let _ = spawn(server);
}

#[actix_web::test]
async fn health_check_works() {
    spawn_app();

    let client = Client::new();

    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to execute requeset.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
