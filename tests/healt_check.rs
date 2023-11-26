use std::net::TcpListener;
use stoic_newsletter::startup::run;

#[tokio::test]
async fn test_health_check_works() {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health-check", app()))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success())
}

#[tokio::test]
async fn test_subscribig_to_newsletter_works() {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/subscribe", app()))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_subscribig_to_newsletter_with_invalid_data_returns_400() {
    let client = reqwest::Client::new();
    let app = app();
    struct TestCase {
        payload: String,
        error: String
    }

    let test_cases = [
        TestCase {payload: "name=le%20guin".to_string(), error: "Parse error: missing field `email`.".to_string()},
        TestCase {payload: "email=ursula_le_guin%40gmail.com".to_string(), error: "Parse error: missing field `name`.".to_string()},
        TestCase {payload: "".to_string(), error: "Parse error: missing field `name`.".to_string()},
    ];

    for case in test_cases {
        let response = client
            .post(format!("{}/subscribe", app))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(case.payload)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status().as_u16(), 400);
        let body = response.text().await.expect("failed to decode request body");
        assert_eq!(body, case.error);
    }

}

fn app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create a tcp listnener");
    let port = listener.local_addr().expect("Unable to get address of listener").port();

    let server = run(listener).expect("Failed to instantiate server");
    tokio::spawn(server);

    return format!("http://127.0.0.1:{}", port);
}
