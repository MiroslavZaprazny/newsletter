use reqwest::Url;
use sqlx::{Connection, PgConnection, PgPool};
use stoic_newsletter::config::{get_config, DatabaseSettings};
use stoic_newsletter::startup::{get_connection_pool, Application};
use uuid::Uuid;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub port: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub fn get_confirmation_link(&self, email_request: &wiremock::Request) -> String {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            links[0].as_str().to_owned()
        };

        let raw_link = &get_link(&body.to_string());
        let mut url = Url::parse(&raw_link).expect("Failed to parse url");
        url.set_port(Some(self.port.parse::<u16>().unwrap()))
            .expect("failed to set port");
        //TODO: figure out why the link has a trailing backslash in tests
        let mut u = url.to_string();
        u.truncate(u.len() - 1);

        u
    }
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
    let port = app.port();
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        port,
        db_pool: get_connection_pool(&config.database),
        email_server,
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
