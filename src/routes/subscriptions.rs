use actix_web::{post, HttpResponse, Responder, web};

#[derive(serde::Deserialize)]
struct SubscribeFormData {
    name: String,
    email: String
}

#[post("/subscribe")]
async fn subscribe(form: web::Form<SubscribeFormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
