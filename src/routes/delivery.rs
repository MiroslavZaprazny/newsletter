use actix_web::http::header::HeaderMap;
use actix_web::{post, web, HttpRequest, HttpResponse};
use argon2::{Argon2, Algorithm, Params};
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::header::{self, HeaderValue};
use sha3::{Digest, Sha3_256};
use sqlx::PgPool;

use crate::{
    domain::{Email, Subscriber, SubscriberName},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
struct DeliveryData {
    content: String,
    subject: String,
}

struct Credentials {
    username: String,
    password: String,
}

enum AuthError {
    InvalidValue,
    MissingHeader,
}

#[tracing::instrument(name = "Sending newsletter to subscribers", skip(body))]
#[post("/delivery")]
async fn delivery(
    body: web::Json<DeliveryData>,
    request: HttpRequest,
    connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let realm = HeaderValue::from_str(r#"Basic realm="delivery""#).unwrap();
    let credentials = match basic_auth(request.headers()) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .insert_header((header::WWW_AUTHENTICATE, realm))
                .finish();
        }
    };

    if validate_credentials(credentials, &connection).await.is_err() {
        return HttpResponse::Unauthorized()
            .insert_header((header::WWW_AUTHENTICATE, realm))
            .finish();
    }

    let subscribers = match get_subscribers(&connection).await {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    for subscriber in subscribers {
        if email_client
            .send_email(subscriber.email, &body.subject, &body.content)
            .await
            .is_err()
        {
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().finish()
}

fn basic_auth(headers: &HeaderMap) -> Result<Credentials, AuthError> {
    let auth_header = match headers.get("Authorization") {
        Some(v) => v,
        None => return Err(AuthError::MissingHeader),
    };

    let auth_header_str = match auth_header.to_str() {
        Ok(v) => v,
        Err(_) => return Err(AuthError::InvalidValue),
    };

    let base64encoded = match auth_header_str.strip_prefix("Basic ") {
        Some(v) => v,
        None => return Err(AuthError::InvalidValue),
    };

    let encoded_bytes = match general_purpose::STANDARD.decode(base64encoded) {
        Ok(v) => v,
        Err(_) => return Err(AuthError::InvalidValue),
    };

    let decoded_credentials = match String::from_utf8(encoded_bytes) {
        Ok(v) => v,
        Err(_) => return Err(AuthError::InvalidValue),
    };

    let mut credentials = decoded_credentials.splitn(2, ":");

    let username = match credentials.next() {
        Some(v) => v.to_string(),
        None => return Err(AuthError::InvalidValue),
    };

    let password = match credentials.next() {
        Some(v) => v.to_string(),
        None => return Err(AuthError::InvalidValue),
    };

    Ok(Credentials { username, password })
}

async fn validate_credentials(credentials: Credentials, connection: &PgPool) -> Result<(), AuthError> {
    let hasher = Argon2::new(
        Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    );
    if sqlx::query!(
        "SELECT id FROM users WHERE username = $1 AND password = $2",
        credentials.username,
    )
        .fetch_one(connection)
        .await.is_err() {
            return Err(AuthError::InvalidValue);
    };

    Ok(())
}

async fn get_subscribers(connection: &PgPool) -> Result<Vec<Subscriber>, sqlx::Error> {
    struct Row {
        name: String,
        email: String,
    }

    let rows = sqlx::query_as!(
        Row,
        "SELECT name, email FROM subscriptions WHERE status = 'confirmed'"
    )
    .fetch_all(connection)
    .await?;

    let subscribers = rows
        .into_iter()
        .map(|row| Subscriber {
            email: Email::parse(row.email).expect("Failed to parse email"),
            name: SubscriberName::parse(row.name).expect("Failed to parse name"),
        })
        .collect();

    Ok(subscribers)
}
