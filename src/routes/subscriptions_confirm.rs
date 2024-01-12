use actix_web::{get, HttpResponse, web::{self, Query}};

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String
}

#[get("/subscription_confirm")]
#[tracing::instrument(
    name = "Confirm a pending subscriber",
)]
pub async fn subscription_confirm(_parameters: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}