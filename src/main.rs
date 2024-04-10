use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::Settings;
use zero2prod::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = Settings::new().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let listener = TcpListener::bind(address).expect("Failed to bind address.");

    run(listener, connection_pool)?.await
}
