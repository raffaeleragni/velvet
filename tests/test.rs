use velvet_web::prelude::*;

mod example_app;
use example_app::app;

#[tokio::test]
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
async fn test_custom_metrics() -> AppResult<()> {
    let server = App::new()
        .route(
            "/",
            get(|| async {
                metric_counter("test_metric_counter").increment(1);
            }),
        )
        .as_test_server()
        .await;
    server.get("/").await;
    assert!(server
        .get("/metrics/prometheus")
        .await
        .text()
        .contains("test_metric_counter"));
    Ok(())
}
