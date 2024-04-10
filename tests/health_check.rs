use actix_web::rt::spawn;
use reqwest::Client;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;

use zero2prod::{configuration::Settings, startup::run};

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // Port 0 is a wildcard port that tells the system to find an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = Settings::new().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let server = run(listener, connection_pool.clone()).expect("Failed to listen");
    // Run it as a background task, because it should not block the test
    let _ = spawn(server);
    TestApp {
        address,
        connection_pool,
    }
}

#[actix_web::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute requeset.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = Client::new();

    let body = "name=optimistic%20snail&email=optimistic_snail%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions limit 1",)
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.name, "optimistic snail");
    assert_eq!(saved.email, "optimistic_snail@gmail.com");
}

#[actix_web::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
