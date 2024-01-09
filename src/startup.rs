use crate::config::{DatabaseSettings, Settings};
use crate::domain::Email;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe, test};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: String,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        // let sender = config.email_client.sender().expect("Could not get parse the sender email");
        let sender =
            Email::parse(String::from("lawsofoutreach@gmail.com")).expect("Failed to parse email");
        let code = String::from("");
        let email_client = EmailClient::new(config.email_client.url, sender, code);

        let connection_pool = get_connection_pool(&config.database);
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address).expect("Failed to create a tcp listnener");
        let port = listener.local_addr().unwrap().port().to_string();

        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> String {
        self.port.to_string()
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPool::connect_lazy(config.connection_string().as_str()).expect("Failed to connect to db")
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscribe)
            .service(test)
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
