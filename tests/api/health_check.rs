use crate::helpers::spawn_app;

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let response = app.get_health_check().await;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
