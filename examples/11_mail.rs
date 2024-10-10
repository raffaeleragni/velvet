use velvet_web::prelude::*;

#[tokio::main]
async fn main() -> AppResult<()> {
    App::new()
        .route("/send", post(send))
        .inject(mailer())
        .start()
        .await?;
    Ok(())
}

async fn send(Extension(mailer): Extension<MailTransport>) -> AppResult<()> {
    let message = MailMessage::builder()
        .from("1@1.com".parse().unwrap())
        .to("2@2.com".parse().unwrap())
        .body("Hello World".to_string())
        .unwrap();
    mailer.send(&message)?;
    Ok(())
}
