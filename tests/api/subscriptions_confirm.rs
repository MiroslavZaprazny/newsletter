use reqwest::Url;
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

    let email_requests = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_requests.body).unwrap();
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    println!("{}", body.to_string());
    let raw_link = &get_link(&body.to_string());
    let mut url = Url::parse(&raw_link).expect("Failed to parse url");
    url.set_port(Some(app.port.parse::<u16>().unwrap()))
        .expect("failed to set port");
    //TODO: figure out why the link has a trailing backslash in tests
    let mut u = url.to_string();
    u.truncate(u.len() - 1);
    assert_eq!(
        u,
        format!("http://127.0.0.1:{}/subscriptions/confirm", app.port)
    );

    let response = client
        .get(u)
        .send()
        .await
        .expect("Failed to send confirmation request");
    assert_eq!(response.status().as_u16(), 200);
}
