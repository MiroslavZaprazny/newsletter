use actix_web::{get, Responder, HttpResponse, http::header::ContentType};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("index.html"))
}
