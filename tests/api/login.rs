use crate::helpers::app;

#[tokio::test]
async fn test_login_with_incorrect_credentials() {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let app = app().await;
    let (username, _) = app.seed_user().await;

    let response = client
        .post(format!("{}/login", app.address))
        .form(&serde_json::json!({
            "username": username,
            "password": String::from("testik")
        }))
        .send()
        .await
        .expect("Failed to send requset to login endpoint");

    assert_eq!(303, response.status().as_u16());
    assert_eq!("/", response.headers()["location"]);
}
