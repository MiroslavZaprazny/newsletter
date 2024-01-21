use chrono::Utc;
use sqlx::Executor;
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::app;

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_form_data() {
    let client = reqwest::Client::new();
    let app = app().await;

    Mock::given(path("mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_subscribe_persists_subscriber() {
    let client = reqwest::Client::new();
    let app = app().await;

    Mock::given(path("mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to send request");

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn test_subscribe_send_confirmation_link() {
    let client = reqwest::Client::new();
    let app = app().await;

    Mock::given(path("mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let email_requests = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_requests.body).unwrap();

    assert!(body
        .to_string()
        .contains("http://127.0.0.1/subscriptions/confirm?subscription_token="));
}

#[tokio::test]
async fn test_subscribing_to_newsletter_with_invalid_data_returns_400() {
    let client = reqwest::Client::new();
    let app = app().await;
    struct TestCase {
        payload: String,
        error: String,
    }

    let test_cases = [
        TestCase {
            payload: "name=le%20guin".to_string(),
            error: "Parse error: missing field `email`.".to_string(),
        },
        TestCase {
            payload: "email=ursula_le_guin%40gmail.com".to_string(),
            error: "Parse error: missing field `name`.".to_string(),
        },
        TestCase {
            payload: "".to_string(),
            error: "Parse error: missing field `name`.".to_string(),
        },
        TestCase {
            payload: format!("name={}&email=testemail", ""),
            error: "".to_string(),
        },
        TestCase {
            payload: format!("name={}&email=testemail", " "),
            error: "".to_string(),
        },
        TestCase {
            payload: format!("name=validnam&email={}", "invalidemail"),
            error: "".to_string(),
        },
    ];

    for case in test_cases {
        let response = client
            .post(format!("{}/subscribe", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(case.payload)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status().as_u16(), 400);
        let body = response
            .text()
            .await
            .expect("failed to decode request body");
        assert_eq!(body, case.error);
    }
}

#[tokio::test]
async fn test_subscribing_with_existing_email_returns_409() {
    let client = reqwest::Client::new();
    let app = app().await;

    sqlx::query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at, status)
        VALUES($1, $2, $3, $4, 'pending_confirmation')
        "#,
        Uuid::new_v4(),
        "test@email.com",
        "test",
        Utc::now()
    )
    .execute(&app.db_pool)
    .await
    .expect("Failed to seed data");

    let response = client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=test%40email.com")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 409);
}
