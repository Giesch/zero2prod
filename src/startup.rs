use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

use crate::configuration::{DatabaseSettings, EmailClientSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::*;
use sqlx::postgres::PgPoolOptions;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let db_pool = get_connection_pool(&config.database)
            .await
            .expect("Failed to connect to Postgres");
        let email_client = email_client(config.email_client);

        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(db_config: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(db_config.with_db())
        .await
}

fn email_client(email_config: EmailClientSettings) -> EmailClient {
    let sender_email = email_config.sender().expect("Invalid sender email address");
    let base_url = email_config
        .base_url()
        .expect("Invalid email client base url");

    EmailClient::new(base_url, sender_email, email_config.authorization_token)
        .expect("Invalid email url path")
}

fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger)
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
