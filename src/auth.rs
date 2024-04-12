pub mod jwt;

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
use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use std::error::Error;

pub struct CookieToken(pub String);
pub struct BearerToken(pub String);

pub trait AuthorizedBearer<F>
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<bool> + 'static,
{
    fn authorized_bearer(self, f: F) -> Self;
}

pub trait AuthorizedCookie<F>
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<bool> + 'static,
{
    fn authorized_cookie(self, f: F) -> Self;
}

impl CookieToken {
    pub fn set_from_claims<T: Serialize>(
        jar: CookieJar,
        claims: T,
    ) -> Result<CookieJar, Box<dyn Error>> {
        let token = jwt::token_from_claims(&claims)?;
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
        let value = jar.get("token").ok_or(response_unauthorized())?.value();
        let value = value.to_string().trim().to_string();
        Ok(Self(value))
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

async fn authorize_from_bearer<F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&str) -> anyhow::Result<bool>,
{
    let (mut parts, body) = request.into_parts();
    let token = match BearerToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(e) => {
            tracing::debug!(?e, "No bearer token in header");
            return response_unauthorized();
        }
    };
    let request = Request::from_parts(parts, body);
    let authorized = f(&token.0);
    match authorized {
        Ok(authorized) => {
            if !authorized {
                return response_unauthorized();
            }
        }
        Err(e) => {
            tracing::debug!(?e, "Failed to verify token");
            return response_unauthorized();
        }
    }
    next.run(request).await
}

async fn authorize_from_cookie<F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&str) -> anyhow::Result<bool>,
{
    let (mut parts, body) = request.into_parts();
    let token = match CookieToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(e) => {
            tracing::debug!(?e, "No bearer token in cookies");
            return response_unauthorized();
        }
    };
    let request = Request::from_parts(parts, body);
    let authorized = f(&token.0);
    match authorized {
        Ok(authorized) => {
            if !authorized {
                return response_unauthorized();
            }
        }
        Err(e) => {
            tracing::debug!(?e, "Failed to verify token");
            return response_unauthorized();
        }
    }
    next.run(request).await
}

impl<F> AuthorizedBearer<F> for Router
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<bool> + 'static,
{
    fn authorized_bearer(self, f: F) -> Self {
        let wrapper = move |r, n| authorize_from_bearer(r, n, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl<F> AuthorizedCookie<F> for Router
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<bool> + 'static,
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
