use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).start().await.unwrap();
}

async fn index() -> impl IntoResponse {
    "Hello World"
}
