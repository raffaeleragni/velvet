use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).start().await.unwrap();
}

async fn index() -> AppResult<impl IntoResponse> {
    metric_counter("counter").increment(1);
    Ok("Hello World")
}
