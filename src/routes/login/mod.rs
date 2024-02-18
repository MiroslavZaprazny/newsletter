use actix_web::{Responder, get, HttpResponse, http::header::ContentType, post, web, ResponseError, error::ErrorUnauthorized};
use reqwest::{header::LOCATION, StatusCode};
use sqlx::PgPool;

use crate::auth::{Credentials, validate_credentials};

#[derive(serde::Deserialize)]
struct FormData {
    username: String,
    password: String,
}

#[get("/login")]
async fn login_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("login.html"))
}

#[post("/login")]
async fn login(
    form: web::Form<FormData>,
    connection: web::Data<PgPool>
) -> impl Responder {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password
    };

    let user_id = match validate_credentials(credentials, &connection).await {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::Unauthorized().insert_header((LOCATION, "/")).finish(),
    };

    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}
