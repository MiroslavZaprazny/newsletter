use std::net::TcpListener;

use ::stoic_newsletter::startup::run;
use sqlx::PgPool;
use stoic_newsletter::{
    config::get_config,
    domain::Email,
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("Failed to retrieve app configuration");
    let connection_pool = PgPool::connect_lazy(&config.database.connection_string())
        .expect("Failed to connect to db");
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address).expect("Failed to create a tcp listnener");
    // let sender = config.email_client.sender().expect("Could not get parse the sender email");

    let sender =
        Email::parse(String::from("lawsofoutreach@gmail.com")).expect("Failed to parse email");
    let code = String::from("");
    let email_client = EmailClient::new(config.email_client.url, sender, code);
    init_subscriber(get_subscriber());

    run(listener, connection_pool, email_client)?.await
}
