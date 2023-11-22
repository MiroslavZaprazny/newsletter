use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/health-check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

pub async fn run() -> std::io::Result<()> {
    return HttpServer::new(|| {
        App::new()
            .service(health_check)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
