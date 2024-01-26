use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::app;

#[tokio::test]
async fn test_subscription_confirm_is_rejected_without_token() {
    let client = reqwest::Client::new();
    let app = app().await;

    let response = client
        .get(format!("{}/subscriptions/confirm", app.address))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn test_click_on_the_confirmation_link_confirms_the_subscriber() {
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

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let link = app.get_confirmation_link(&email_request);
    assert!(link.contains(&format!(
        "http://127.0.0.1:{}/subscriptions/confirm?subscription_token=",
        app.port
    )));

    client
        .get(link)
        .send()
        .await
        .expect("Failed to send confirmation request");

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch subscriber");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn test_confirmation_link_returns_200_when_called() {
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

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let link = app.get_confirmation_link(&email_request);
    assert!(link.contains(&format!(
        "http://127.0.0.1:{}/subscriptions/confirm?subscription_token=",
        app.port
    )));

    let response = client
        .get(link)
        .send()
        .await
        .expect("Failed to send confirmation request");
    assert_eq!(response.status().as_u16(), 200);
}
