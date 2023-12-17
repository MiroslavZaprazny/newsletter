use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
struct SubscribeFormData {
    name: String,
    email: String,
}

#[post("/subscribe")]
async fn subscribe(
    form: web::Form<SubscribeFormData>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let req_id = Uuid::new_v4();
    let req_span = tracing::info_span!("Adding a new subscriber", %req_id, subscriber_name = %form.name, subscriber_email = %form.email);
    let _span_guard = req_span.enter();

    let query_span = tracing::info_span!("Inserting a new subscriber into database");
    return match sqlx::query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at)
        VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("request_id: {} - Subscriber details have been saved", req_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("request_id: {} - Failed to save details {:?} {:?}", req_id, form, e);
            HttpResponse::InternalServerError().finish()
        }
    };
}
