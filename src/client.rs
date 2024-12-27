/// Create a new http client to inject into the application.
/// Use with the extractor Extension<Client>
pub fn client() -> reqwest::Client {
    reqwest::Client::default()
}
