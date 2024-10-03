use serial_test::serial;
use velvet_web::prelude::*;

mod example_app;
use example_app::app;

#[tokio::test]
#[serial]
async fn test() -> AppResult<()> {
    let server = app().await?.as_test_server().await;
    assert_eq!("OK", server.get("/").await.text());
    assert_eq!("static\n", server.get("/static.txt").await.text());
    assert!(server
        .get("/metrics/prometheus")
        .await
        .text()
        .contains("axum_http_requests"));
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_custom_metrics() -> AppResult<()> {
    let server = App::new()
        .route(
            "/",
            get(|| async {
                metric_counter("test_metric_counter").increment(1);
                metric_counter("another_metric").increment(1);
            }),
        )
        .as_test_server()
        .await;
    server.get("/").await;
    let result = server.get("/metrics/prometheus").await.text();
    assert!(result.contains("test_metric_counter"));
    assert!(result.contains("another_metric"));
    Ok(())
}
