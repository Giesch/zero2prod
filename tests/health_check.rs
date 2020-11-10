use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::env;
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

lazy_static::lazy_static! {
    static ref TRACING: () = {
        let filter = if env::var("TEST_LOG").is_ok() { "debug" } else { "" };
        let subscriber = get_subscriber("test".into(), filter.into());
        init_subscriber(subscriber);
    };
}

async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = get_configuration().expect("Failed to read config");
    config.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_db(&config.database).await;

    let server = startup::run(listener, db_pool.clone()).expect("failed to run server");

    // This is torn down along with the runtime by the actix_rt::test macro
    let _ = tokio::spawn(server);

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

#[actix_rt::test]
async fn health_check_works() {
    let TestApp { address, .. } = spawn_app().await;

    let client = reqwest::Client::new();
    let route = format!("{}/health_check", &address);
    let response = client
        .get(&route)
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_200_for_valid_form() {
    let TestApp { address, db_pool } = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_400_for_missing_data() {
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 when {}",
            error_msg
        );
    }
}
