use crate::prelude::AppResult;
use askama::Template;
use lettre::{
    message::MultiPart,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    Message, SmtpTransport, Transport,
};
use std::env;
use tracing::warn;

pub fn mailer() -> SmtpTransport {
    dotenvy::dotenv().ok();
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    env::var("MAIL_FROM").expect("env var MAIL_FROM required to setup mail system");
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

#[derive(Template)]
#[template(path = "mail_confirm.html")]
struct HtmlMail {
    username: String,
    code: String,
    link: String,
    site: String,
}

#[derive(Template)]
#[template(path = "mail_confirm.txt")]
struct TextMail {
    username: String,
    code: String,
    link: String,
    site: String,
}

pub fn send_confirmation_email(
    mailer: crate::prelude::MailTransport,
    host: &str,
    link: &str,
    username: &str,
    email: &str,
    code: &str,
) -> AppResult<()> {
    let Ok(from) = env::var("MAIL_FROM") else {
        return Err("no MAIL_FROM env found".into());
    };
    let plain: String = TextMail {
        username: username.to_string(),
        code: code.to_string(),
        site: host.to_string(),
        link: link.to_string(),
    }
    .to_string();
    let html = HtmlMail {
        username: username.to_string(),
        code: code.to_string(),
        site: host.to_string(),
        link: link.to_string(),
    }
    .to_string();
    let message = Message::builder()
        .from(from.parse()?)
        .to(email.parse()?)
        .multipart(MultiPart::alternative_plain_html(plain, html))
        .expect("Could not build email");
    mailer.send(&message)?;
    Ok(())
}
