use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use stoic_newsletter::config::{get_config, DatabaseSettings};
use stoic_newsletter::startup::run;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn test_health_check_works() {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health-check", app().await.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success())
}

#[tokio::test]
async fn test_subscribig_to_newsletter_works() {
    let client = reqwest::Client::new();
    let app = app().await;
    let response = client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn test_subscribig_to_newsletter_with_invalid_data_returns_400() {
    let client = reqwest::Client::new();
    let app = app().await;
    struct TestCase {
        payload: String,
        error: String,
    }

    let test_cases = [
        TestCase {
            payload: "name=le%20guin".to_string(),
            error: "Parse error: missing field `email`.".to_string(),
        },
        TestCase {
            payload: "email=ursula_le_guin%40gmail.com".to_string(),
            error: "Parse error: missing field `name`.".to_string(),
        },
        TestCase {
            payload: "".to_string(),
            error: "Parse error: missing field `name`.".to_string(),
        },
    ];

    for case in test_cases {
        let response = client
            .post(format!("{}/subscribe", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(case.payload)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status().as_u16(), 400);
        let body = response
            .text()
            .await
            .expect("failed to decode request body");
        assert_eq!(body, case.error);
    }
}

async fn app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create a tcp listnener");
    let port = listener
        .local_addr()
        .expect("Unable to get address of listener")
        .port();
    let mut config = get_config().expect("Failed to retrieve app configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_db(&config.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to instantiate server");
    tokio::spawn(server);

    return TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    };
}

pub async fn configure_db(settings: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&settings.connection_string_without_db())
        .await
        .expect("failed to connect to db");

    sqlx::query(format!(r#"CREATE DATABASE "{}";"#, settings.database_name).as_str())
        .execute(&mut connection)
        .await
        .expect("Failed to create test db");

    let connection_pool = PgPool::connect(&settings.connection_string())
        .await
        .expect("Failed to connect to db");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to execute migrations");

    return connection_pool;
}
