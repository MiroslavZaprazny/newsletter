use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use stoic_newsletter::config::{get_config, DatabaseSettings};
use stoic_newsletter::email_client::EmailClient;
use stoic_newsletter::startup::run;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create a tcp listnener");
    let port = listener
        .local_addr()
        .expect("Unable to get address of listener")
        .port();
    let mut config = get_config().expect("Failed to retrieve app configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_db(&config.database).await;
    let sender = config
        .email_client
        .sender()
        .expect("Could not get parse the sender email");
    let auth_code = String::from("authcode123");
    let email_client = EmailClient::new(config.email_client.url, sender, auth_code);

    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to instantiate server");
    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

pub async fn configure_db(settings: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&settings.without_db())
        .await
        .expect("failed to connect to db");

    sqlx::query(format!(r#"CREATE DATABASE "{}";"#, settings.database_name).as_str())
        .execute(&mut connection)
        .await
        .expect("Failed to create test db");

    let connection_pool = PgPool::connect(&settings.without_db())
        .await
        .expect("Failed to connect to db");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to execute migrations");

    connection_pool
}
