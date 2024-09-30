use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new()
        .route("/", get(index))
        .inject(client())
        .start()
        .await;
}

async fn index(Extension(client): Extension<Client>) -> AppResult<impl IntoResponse> {
    Ok(client
        .get("https://en.wikipedia.org")
        .send()
        .await?
        .text()
        .await?)
}
