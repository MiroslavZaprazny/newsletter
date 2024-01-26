use crate::helpers::app;

#[tokio::test]
async fn test_delivering_emails_to_subscribers_works() {
    let client = reqwest::Client::new();
    let app = app().await;
    app.seed_subscriber().await;

    let response = client
        .post(format!("{}/delivery", app.address))
        .body("subject=test&content=testik")
        .send()
        .await
        .expect("Failed to submit a send request");

    assert_eq!(response.status().as_u16(), 200)
}
