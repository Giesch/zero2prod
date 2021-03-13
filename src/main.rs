use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use zero2prod::configuration::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");

    let db_pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(config.database.with_db())
        .await
        .expect("Failed to connect to postgres");

    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let base_url = config
        .email_client
        .base_url()
        .expect("Invalid email client base url");
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        config.email_client.authorization_token,
    )
    .expect("Invalid email url path");

    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool, email_client)?.await
}
