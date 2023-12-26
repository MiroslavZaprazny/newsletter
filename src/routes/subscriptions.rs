use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Subscriber, SubscriberName};

#[derive(serde::Deserialize, Debug)]
struct SubscribeFormData {
    name: String,
    email: String,
}

#[post("/subscribe")]
#[tracing::instrument(
    name = "Adding a new subscriber", skip(form, connection),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
async fn subscribe(
    form: web::Form<SubscribeFormData>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let new_subscriber = Subscriber {
        name,
        email: form.0.email,
    };

    return match insert_subscriber(&connection, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to save details {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    };
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at)
        VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}
