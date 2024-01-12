use sqlx::{Connection, PgConnection, PgPool};
use stoic_newsletter::config::{get_config, DatabaseSettings};
use stoic_newsletter::startup::{get_connection_pool, Application};
use uuid::Uuid;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer
}

pub async fn app() -> TestApp {
    let email_server = MockServer::start().await;

    let config = {
        let mut c = get_config().expect("Failed to read config");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.url = email_server.uri();

        c
    };
    configure_db(&config.database).await;

    let app = Application::build(config.clone())
        .await
        .expect("Failed to build app");
    let address = format!("http://127.0.0.1:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&config.database),
        email_server
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

    let connection_pool = PgPool::connect(&settings.connection_string())
        .await
        .expect("Failed to connect to db");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to execute migrations");

    connection_pool
}
