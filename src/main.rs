use std::net::TcpListener;

use ::stoic_newsletter::startup::run;
use sqlx::PgPool;
use stoic_newsletter::{
    config::get_config,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("Failed to retrieve app configuration");
    let connection_pool = PgPool::connect_lazy_with(config.database.with_db());
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address).expect("Failed to create a tcp listnener");
    init_subscriber(get_subscriber());

    run(listener, connection_pool)?.await
}
