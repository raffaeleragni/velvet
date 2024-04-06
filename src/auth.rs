use std::{env, error::Error, str::FromStr};

use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::header::AUTHORIZATION;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::OnceCell;

static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();
static ENCODING_KEY: OnceCell<EncodingKey> = OnceCell::const_new();
pub async fn setup_jwt_key_from_env() {
    dotenv::dotenv().ok();
    let deckey = DecodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref());
    let enckey = EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref());
    DECODING_KEY.get_or_init(|| async move { deckey }).await;
    ENCODING_KEY.get_or_init(|| async move { enckey }).await;
}

pub struct CookieToken(pub String);
pub struct BearerToken(pub String);
pub struct VerifiedClaims<T: DeserializeOwned>(pub Header, pub T);

pub trait AuthorizedBearer<F>
where
    F: Send + Sync + Clone + Fn(&str) -> Result<bool, StatusCode> + 'static,
{
    fn authorized_bearer(self, f: F) -> Self;
}

pub trait AuthorizedCookie<F>
where
    F: Send + Sync + Clone + Fn(&str) -> Result<bool, StatusCode> + 'static,
{
    fn authorized_cookie(self, f: F) -> Self;
}

pub fn claims_for<T: DeserializeOwned>(token: &str) -> Result<T, StatusCode> {
    Ok(token
        .parse::<VerifiedClaims<T>>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .1)
}

impl CookieToken {
    pub fn set_from_claims<T: Serialize>(
        jar: CookieJar,
        claims: T,
    ) -> Result<CookieJar, Box<dyn Error>> {
        let key = ENCODING_KEY
            .get()
            .ok_or("ENCODING_KEY was not initialized")?;
        let token = encode(&Header::default(), &claims, key)?;
        Ok(CookieToken::set(jar, token))
    }

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
    type Rejection = Response;

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
    type Rejection = Response;

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

async fn authorize_from_bearer<F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&str) -> Result<bool, StatusCode>,
{
    let (mut parts, body) = request.into_parts();
    let token = match BearerToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(_) => return response_unauthorized(),
    };
    let request = Request::from_parts(parts, body);
    let authorized = f(&token.0);
    match authorized {
        Ok(authorized) => {
            if !authorized {
                return response_unauthorized();
            }
        }
        Err(_) => return response_unauthorized(),
    }
    next.run(request).await
}

async fn authorize_from_cookie<F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&str) -> Result<bool, StatusCode>,
{
    let (mut parts, body) = request.into_parts();
    let token = match CookieToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(_) => return response_unauthorized(),
    };
    let request = Request::from_parts(parts, body);
    let authorized = f(&token.0);
    match authorized {
        Ok(authorized) => {
            if !authorized {
                return response_unauthorized();
            }
        }
        Err(_) => return response_unauthorized(),
    }
    next.run(request).await
}

impl<F> AuthorizedBearer<F> for Router
where
    F: Send + Sync + Clone + Fn(&str) -> Result<bool, StatusCode> + 'static,
{
    fn authorized_bearer(self, f: F) -> Self {
        let wrapper = move |r, n| authorize_from_bearer(r, n, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl<F> AuthorizedCookie<F> for Router
where
    F: Send + Sync + Clone + Fn(&str) -> Result<bool, StatusCode> + 'static,
{
    fn authorized_cookie(self, f: F) -> Self {
        let wrapper = move |r, n| authorize_from_cookie(r, n, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

fn response_unauthorized() -> Response {
    Response::builder()
        .status(401)
        .body(Body::new("401 Unauthorized".to_string()))
        .unwrap()
}
