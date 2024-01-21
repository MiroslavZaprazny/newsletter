use crate::{
    domain::{Email, Subscriber, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

enum SubscriberError {
    DuplicateEmail,
    DatabaseFailure,
}

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

    let mut transaction = match connection.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(e) => {
            if let SubscriberError::DuplicateEmail = e {
                return HttpResponse::Conflict().finish();
            }
            return HttpResponse::InternalServerError().finish();
        }
    };
    let token = generate_subscription_token();
    if store_token(&mut transaction, subscriber_id, &token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let res = send_confirmation_email(&email_client, subscriber, &base_url.0, &token).await;
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
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
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
    skip(form, transaction)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    form: &Subscriber,
) -> Result<Uuid, SubscriberError> {
    let subscriber_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at, status)
        VALUES($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    )
    .execute(&mut **transaction)
    .await;

    if let Err(sqlx::Error::Database(e)) = result {
        tracing::error!("Failed to execute query {:?}", e);
        if *UNIQUE_CONSTRAINT_VIOLATION_CODE.to_string() == e.code().unwrap() {
            return Err(SubscriberError::DuplicateEmail);
        }

        return Err(SubscriberError::DatabaseFailure);
    }

    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(name = "Saving the subscription token")]
async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES($1, $2)"#,
        token,
        subscriber_id
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to save subscription_token");
        e
    })?;

    Ok(())
}
