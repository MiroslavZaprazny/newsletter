use crate::helpers::app;

#[tokio::test]
async fn test_subscription_confirm_is_rejected_without_token() {
    let client = reqwest::Client::new();
    let app = app().await;

    let response = client
        .get(format!("{}/subscription_confirm", app.address))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 400);
}
