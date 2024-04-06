use std::{env, error::Error, str::FromStr};

use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::{self, Next},
    response::Response,
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{decode, DecodingKey, Header, Validation};
use reqwest::header::AUTHORIZATION;
use serde::de::DeserializeOwned;
use tokio::sync::OnceCell;

static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();
pub async fn setup_jwt_key_from_env() {
    dotenv::dotenv().ok();
    let key = DecodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref());
    DECODING_KEY.get_or_init(|| async move { key }).await;
}

pub struct CookieToken(pub String);
pub struct BearerToken(pub String);
pub struct VerifiedClaims<T: DeserializeOwned>(pub Header, pub T);

pub trait AuthorizedBearer<T, F>
where
    F: Send + Sync + Clone + Fn(&T) -> bool + 'static,
    T: DeserializeOwned,
{
    fn authorized_bearer(self, f: F) -> Self;
}

pub trait AuthorizedCookie<T, F>
where
    F: Send + Sync + Clone + Fn(&T) -> bool + 'static,
    T: DeserializeOwned,
{
    fn authorized_cookie(self, f: F) -> Self;
}

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

async fn authorize_from_token_string<T, F>(
    request: Request,
    next: Next,
    token: String,
    f: F,
) -> Response
where
    F: Fn(&T) -> bool,
    T: DeserializeOwned,
{
    let claims: VerifiedClaims<T> = match token.parse() {
        Ok(claims) => claims,
        Err(_) => return response_unauthorized(),
    };

    let authorized = f(&claims.1);
    if !authorized {
        return response_unauthorized();
    }

    let response = next.run(request).await;

    response
}

async fn authorize_from_bearer<T, F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&T) -> bool,
    T: DeserializeOwned,
{
    let (mut parts, body) = request.into_parts();
    let token = match BearerToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(_) => return response_unauthorized(),
    };
    let request = Request::from_parts(parts, body);
    authorize_from_token_string(request, next, token.0, f).await
}

async fn authorize_from_cookie<T, F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&T) -> bool,
    T: DeserializeOwned,
{
    let (mut parts, body) = request.into_parts();
    let token = match CookieToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(_) => return response_unauthorized(),
    };
    let request = Request::from_parts(parts, body);
    authorize_from_token_string(request, next, token.0, f).await
}

async fn otherlayer<F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn() -> bool,
{
    if f() {
        println!("a");
    }
    next.run(request).await
}

impl<T, F> AuthorizedBearer<T, F> for Router
where
    F: Send + Sync + Clone + Fn(&T) -> bool + 'static,
    T: DeserializeOwned,
{
    fn authorized_bearer(self, f: F) -> Self {
        let ff = || true;
        let wrapper = |r, n| otherlayer(r, n, ff.clone());
        self.layer(middleware::from_fn(wrapper));
        let wrapper = |r, n| authorize_from_bearer(r, n, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl<T, F> AuthorizedCookie<T, F> for Router
where
    F: Send + Sync + Clone + Fn(&T) -> bool + 'static,
    T: DeserializeOwned,
{
    fn authorized_cookie(self, f: F) -> Self {
        let wrapper = |r, n| authorize_from_cookie(r, n, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

fn response_unauthorized() -> Response {
    Response::builder()
        .status(401)
        .body(Body::new("401 Unauthorized".to_string()))
        .unwrap()
}
