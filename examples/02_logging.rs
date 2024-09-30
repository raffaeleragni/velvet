use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).start().await;
}

async fn index() -> AppResult<impl IntoResponse> {
    info!("Logging some info");
    Ok("Hello World")
}
