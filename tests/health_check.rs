use actix_web::rt::spawn;
use once_cell::sync::Lazy;
use reqwest::Client;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

// Ensure that the tracing stack is only initialised once using `once_call`
static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test".to_string();
    let default_filter_level = "info".into();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

use zero2prod::{
    configuration::{DatabaseSettings, Settings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Port 0 is a wildcard port that tells the system to find an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = Settings::new().expect("Failed to read configuration.");
    // Create a random database for test isolation
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to listen");
    // Run it as a background task, because it should not block the test
    let _ = spawn(server);
    TestApp {
        address,
        connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // To use connect() function, you must import sqlx::Connection trait
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    // To use execute() method, you must import sqlx::Executor trait
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
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
