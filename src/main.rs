#[actix_web::main]
async fn main() -> std::io::Result<()> {
    stoic_newsletter::run().await
}

