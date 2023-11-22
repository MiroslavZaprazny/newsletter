#[tokio::test]
async fn test_health_check_works() {
    app().await.expect("Failed to instantiate App");
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1/health-check")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success())
}

async fn app() -> std::io::Result<()> {
    return stoic_newsletter::run().await
}
