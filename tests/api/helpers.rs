use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::env;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

lazy_static::lazy_static! {
    static ref TRACING: () = {
        let filter = if env::var("TEST_LOG").is_ok() { "debug" } else { "" };
        let subscriber = get_subscriber("test".into(), filter.into());
        init_subscriber(subscriber);
    };
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_health_check(&self) -> reqwest::Response {
        let route = format!("{}/health_check", &self.address);
        reqwest::Client::new()
            .get(&route)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);

    let config = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_db(&config.database).await;

    let app = Application::build(config.clone())
        .await
        .expect("Failed to build application");
    let address = format!("http://localhost:{}", app.port());
    // This is torn down along with the runtime by the actix_rt::test macro
    let _ = tokio::spawn(app.run_until_stopped());

    let db_pool = startup::get_connection_pool(&config.database)
        .await
        .expect("Failed to connect to database");

    TestApp { address, db_pool }
}

async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to postgres");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}""#, config.database_name))
        .await
        .expect("Failed to create test database");

    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    db_pool
}
