use crate::helpers::app;

#[tokio::test]
async fn test_health_check_works() {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health-check", app().await.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success())
}

