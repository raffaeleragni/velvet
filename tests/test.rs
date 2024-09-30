use velvet_web::prelude::*;

mod example_app;
use example_app::app;

#[tokio::test]
async fn test() -> AppResult<()> {
    let server = app().await?.as_test_server();
    assert_eq!("OK", server.get("/").await.text());

    Ok(())
}
