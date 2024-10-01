use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    let app = App::new().route("/", get(|| async { "result" }));
    let server = app.as_test_server().await;
    let response = server.get("/").await.text();
    assert_eq!(response, "result");
}
