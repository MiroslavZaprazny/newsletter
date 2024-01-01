use crate::{
    domain::{Email, Subscriber, SubscriberName},
    email_client::EmailClient,
};
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
struct SubscribeFormData {
    name: String,
    email: String,
}

impl TryFrom<SubscribeFormData> for Subscriber {
    type Error = String;
    fn try_from(value: SubscribeFormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = Email::parse(value.email)?;

        Ok(Self { name, email })
    }
}

#[post("/test")]
async fn test(email_client: web::Data<EmailClient>) -> impl Responder {
    let recipient =
        Email::parse(String::from("miro.zaprazny8@gmail.com")).expect("Failed to parse email");
    let res = email_client
        .send_email(recipient, "test email", "testing")
        .await;

    return HttpResponse::Ok().finish();
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
    let subscriber = match Subscriber::try_from(form.0) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    return match insert_subscriber(&connection, &subscriber).await {
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
        form.email.as_ref(),
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
