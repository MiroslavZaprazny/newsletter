use actix_web::{post, web, HttpRequest, HttpResponse};
use reqwest::header::{self, HeaderValue};
use sqlx::PgPool;

use crate::auth::{basic_auth, validate_credentials};
use crate::{
    domain::{Email, Subscriber, SubscriberName},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
struct DeliveryData {
    content: String,
    subject: String,
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

    if validate_credentials(credentials, &connection)
        .await
        .is_err()
    {
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
