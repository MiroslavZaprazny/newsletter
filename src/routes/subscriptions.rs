use crate::{
    domain::{Email, Subscriber, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
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
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> impl Responder {
    let subscriber = match Subscriber::try_from(form.0) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&connection, &subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let res = send_confirmation_email(&email_client, subscriber, &base_url.0).await;
    if res.is_err() {
        tracing::error!("Failed to send email {:?}", res);
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok().finish()
}

async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber: Subscriber,
    base_url: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm", base_url);
    let body = format!(
        "Welcome to our newsletter <br> Click <a href=\"{}\"here</a> to confirm the subscription",
        confirmation_link
    );

    email_client
        .send_email(subscriber.email, "Newsletter subscription", &body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at, status)
        VALUES($1, $2, $3, $4, 'pending_confirmation')
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
