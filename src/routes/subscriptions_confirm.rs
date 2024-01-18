use actix_web::{
    get,
    web::{self, Query},
    HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[get("/subscriptions/confirm")]
#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn subscription_confirm(
    parameters: web::Query<Parameters>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    let id = match get_subscriber_id_from_token(&connection, parameters.0.subscription_token).await
    {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match id {
        None => return HttpResponse::Unauthorized().finish(),
        Some(id) => {
            if confirm_subscriber(&connection, id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    return HttpResponse::Ok().finish();
}

#[tracing::instrument(name = "Fech subscriber by token")]
async fn get_subscriber_id_from_token(
    connection: &PgPool,
    subscription_token: String,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1",
        subscription_token
    )
    .fetch_optional(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fech subscriber");
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Fech subscriber by token")]
async fn confirm_subscriber(connection: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE subscriptions SET status = 'confirmed' WHERE id = $1",
        subscriber_id
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to confirm subscriber");
        e
    })?;

    Ok(())
}
