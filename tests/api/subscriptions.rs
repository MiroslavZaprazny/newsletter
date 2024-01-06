use crate::helpers::app;

#[tokio::test]
async fn test_subscribing_to_newsletter_works() {
    let client = reqwest::Client::new();
    let app = app().await;
    let response = client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
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
