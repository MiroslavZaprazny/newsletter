use std::net::TcpListener;

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, web};
use actix_web::dev::Server;

#[get("/health-check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct SubscribeFormData {
    name: String,
    email: String
}

#[post("/subscribe")]
async fn subscribe(form: web::Form<SubscribeFormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(subscribe)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
