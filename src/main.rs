use stoic_newsletter::{
    config::get_config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("Failed to retrieve app configuration");
    init_subscriber(get_subscriber());

    let app = Application::build(config).await?;
    app.run_until_stopped().await?;

    Ok(())
}
