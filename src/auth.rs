use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::PgPool;
use actix_web::http::header::HeaderMap;
use base64::{engine::general_purpose, Engine};

pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub enum AuthError {
    InvalidValue,
    MissingHeader,
}

pub async fn validate_credentials(
    credentials: Credentials,
    connection: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let row = sqlx::query!(
        "SELECT id, password FROM users WHERE username = $1",
        credentials.username,
    )
    .fetch_one(connection)
    .await;

    let (id, password) = match row {
        Ok(r) => (r.id, r.password),
        Err(_) => return Err(AuthError::InvalidValue),
    };

    let password = match PasswordHash::new(&password) {
        Ok(v) => v,
        Err(_) => return Err(AuthError::InvalidValue),
    };

    if Argon2::default()
        .verify_password(credentials.password.as_bytes(), &password)
        .is_err()
    {
        return Err(AuthError::InvalidValue);
    }

    Ok(id)
}


pub fn basic_auth(headers: &HeaderMap) -> Result<Credentials, AuthError> {
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

    let mut credentials = decoded_credentials.splitn(2, ':');

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
