use std::{error::Error, str::FromStr};

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{decode, DecodingKey, Header, Validation};
use reqwest::header::AUTHORIZATION;
use serde::de::DeserializeOwned;
use tokio::sync::OnceCell;

pub static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();
pub struct CookieToken(pub String);
pub struct BearerToken(pub String);
pub struct VerifiedClaims<T: DeserializeOwned>(pub Header, pub T);

impl CookieToken {
    pub fn set(jar: CookieJar, token: String) -> CookieJar {
        let c = Cookie::build(("token", token))
            .secure(true)
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .build();
        jar.add(c)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for CookieToken
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response<String>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(err) => match err {},
        };
        let value = jar.get("token").ok_or(response_unauthorized())?;
        Ok(Self(value.to_string()))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for BearerToken
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response<String>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(response_unauthorized())?
            .to_str()
            .map_err(|_| response_unauthorized())?;
        let split = header_value.split_once(' ');
        match split {
            Some(("Bearer", value)) => Ok(Self(value.to_string())),
            _ => Err(response_unauthorized()),
        }
    }
}

impl<T: DeserializeOwned> FromStr for VerifiedClaims<T> {
    type Err = Box<dyn Error>;
    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let key = DECODING_KEY
            .get()
            .ok_or("DECODING_KEY was not initialized")?;
        let decoded = decode::<T>(token, key, &Validation::default())?;
        Ok(VerifiedClaims(decoded.header, decoded.claims))
    }
}

fn response_unauthorized() -> axum::response::Response<String> {
    axum::response::Response::builder()
        .status(401)
        .body("401 Unauthorized".to_string())
        .unwrap()
}
