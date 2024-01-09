use sqlx::{Connection, PgConnection, PgPool};
use stoic_newsletter::config::{get_config, DatabaseSettings};
use stoic_newsletter::startup::{get_connection_pool, Application};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn app() -> TestApp {
    let config = {
        let mut c = get_config().expect("Failed to read config");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;

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
