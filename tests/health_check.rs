use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();
    let route = format!("{}/health_check", &address);
    let response = client
        .get(&route)
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("failed to run server");

    // This is torn down along with the runtime by the actix_rt::test macro
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
