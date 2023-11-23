use std::net::TcpListener;

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

fn app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create a tcp listnener");
    let port = listener.local_addr().expect("Unable to get address of listener").port();

    let server = stoic_newsletter::run(listener).expect("Failed to instantiate server");
    tokio::spawn(server);

    return format!("http://127.0.0.1:{}", port);
}
