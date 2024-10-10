use std::env;

use lettre::{
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    SmtpTransport,
};
use tracing::warn;

pub fn mailer() -> SmtpTransport {
    dotenvy::dotenv().ok();
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let host = env::var("MAIL_HOST").expect("env var MAIL_HOST required to setup mail system");
    let mut tls = TlsParameters::builder(host.clone());
    if let Ok(true) =
        env::var("MAIL_ACCEPT_INVALID_CERTS").map(|s| s.parse::<bool>().unwrap_or(false))
    {
        warn!("Accepting invalid certs for smtp, use only for dev");
        tls = tls
            .dangerous_accept_invalid_certs(true)
            .dangerous_accept_invalid_hostnames(true);
    }
    let mut mailer = SmtpTransport::builder_dangerous(host.as_str())
        .tls(Tls::Wrapper(tls.build().unwrap()))
        .port(465);
    if let Ok(port) = env::var("MAIL_PORT") {
        mailer = mailer.port(port.parse().unwrap());
    }
    if let Ok(username) = env::var("MAIL_USERNAME") {
        if let Ok(password) = env::var("MAIL_PASSWORD") {
            mailer = mailer.credentials(Credentials::new(username, password));
        }
    }
    mailer.build()
}
