use zero2prod::test_utils::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let url = format!("{}/health_check", app.address);
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to execute request.");

    dbg!(&response);

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
