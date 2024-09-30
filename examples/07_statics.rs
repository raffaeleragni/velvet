use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    #[derive(RustEmbed)]
    #[folder = "statics"]
    struct S;

    App::new().statics::<S>().start().await;
}
