use actix_web::{post, HttpResponse, web};
use sqlx::PgPool;

use crate::domain::Subscriber;

#[derive(serde::Deserialize)]
struct DeliveryData {
    content: String
}

#[post("/delivery")]
async fn delivery(
    body: web::Form<DeliveryData>,
    connection: web::Data<PgPool>

) -> HttpResponse {
    HttpResponse::Ok().finish() 
}

async fn get_subscribers(connection: PgPool) {
    let subscribers = sqlx::query_as!(
        Subscriber,
        "SELECT email FROM subscrtiptions WHERE status = confirmed"
    ).fetch_all(&mut connection)
    .await;
}
