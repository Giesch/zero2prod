use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");

    let db_pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool)?.await
}
