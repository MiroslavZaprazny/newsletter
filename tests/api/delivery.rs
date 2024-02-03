use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::app;

#[tokio::test]
async fn test_delivering_emails_to_subscribers_works() {
    let client = reqwest::Client::new();
    let app = app().await;
    app.seed_subscriber(String::from("confirmed")).await;
    let (username, password) = app.seed_user().await;

    Mock::given(path("mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = client
        .post(format!("{}/delivery", app.address))
        .json(&serde_json::json!({
            "subject": "jako",
            "content": "jako",
        }))
        .basic_auth(username, Some(password))
        .send()
        .await
        .expect("Failed to submit a send request");

    assert_eq!(response.status().as_u16(), 200)
}

#[tokio::test]
async fn test_request_without_auth_token_are_rejected() {
    let client = reqwest::Client::new();
    let app = app().await;

    let response = client
        .post(format!("{}/delivery", app.address))
        .json(&serde_json::json!({
            "subject": "jako",
            "content": "jako",
        }))
        .send()
        .await
        .expect("Failed to submit a send request");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="delivery""#,
        response.headers()["WWW-Authenticate"]
    );
}
