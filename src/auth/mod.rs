pub mod jwt;
#[cfg(feature = "login")]
pub mod login;

use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::{self, Next},
    response::IntoResponse,
    response::{Redirect, Response},
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jwt::claims_for;
use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use std::error::Error;

pub struct CookieToken(pub String);
pub struct BearerToken(pub String);

pub struct CookieClaims<T>(pub T);
pub struct BearerClaims<T>(pub T);

pub enum AuthResult {
    OK,
    Unauthorized,
    Redirect(String),
}

impl From<bool> for AuthResult {
    fn from(value: bool) -> Self {
        if value {
            return AuthResult::OK;
        }
        AuthResult::Unauthorized
    }
}

pub trait AuthorizedBearer<F>
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_bearer(self, f: F) -> Self;
}

pub trait AuthorizedBearerWithClaims<T, FT>
where
    T: DeserializeOwned,
    FT: Send + Sync + Clone + Fn(T) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_bearer_claims(self, f: FT) -> Self;
}

pub trait AuthorizedBearerWithRole {
    fn authorized_bearer_role(self, role: String) -> Self;
}

pub trait AuthorizedCookie<F>
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_cookie(self, redirect_to_login: &'static str, f: F) -> Self;
}

pub trait AuthorizedCookieWithClaims<T, FT>
where
    T: DeserializeOwned,
    FT: Send + Sync + Clone + Fn(T) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_cookie_claims(self, redirect_to_login: &'static str, f: FT) -> Self;
}

pub trait AuthorizedCookieWithRole {
    fn authorized_cookie_role(self, redirect_to_login: &'static str, role: &'static str) -> Self;
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

    pub fn remove(jar: CookieJar) -> CookieJar {
        jar.remove(Cookie::build("token"))
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
impl<S, T> FromRequestParts<S> for CookieClaims<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = CookieToken::from_request_parts(parts, state).await?;
        let claims = claims_for::<T>(token.0.as_str()).map_err(|_| response_unauthorized())?;
        Ok(CookieClaims::<T>(claims))
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

#[async_trait]
impl<S, T> FromRequestParts<S> for BearerClaims<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = CookieToken::from_request_parts(parts, state).await?;
        let claims = claims_for::<T>(token.0.as_str()).map_err(|_| response_unauthorized())?;
        Ok(BearerClaims::<T>(claims))
    }
}

async fn authorize_from_bearer<F>(request: Request, next: Next, f: F) -> Response
where
    F: Fn(&str) -> anyhow::Result<AuthResult>,
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
        Ok(authorized) => match authorized {
            AuthResult::Unauthorized => return response_unauthorized(),
            AuthResult::Redirect(target) => return Redirect::to(target.as_str()).into_response(),
            AuthResult::OK => (),
        },
        Err(e) => {
            tracing::debug!(?e, "Failed to verify token");
            return response_unauthorized();
        }
    }
    next.run(request).await
}

async fn authorize_from_cookie<F>(
    request: Request,
    next: Next,
    redirect: &'static str,
    f: F,
) -> Response
where
    F: Fn(&str) -> anyhow::Result<AuthResult>,
{
    let (mut parts, body) = request.into_parts();
    let token = match CookieToken::from_request_parts(&mut parts, &()).await {
        Ok(token) => token,
        Err(e) => {
            tracing::debug!(?e, "No bearer token in cookies");
            return Redirect::to(redirect).into_response();
        }
    };
    let request = Request::from_parts(parts, body);
    let authorized = f(&token.0);
    match authorized {
        Ok(authorized) => match authorized {
            AuthResult::Unauthorized => return Redirect::to(redirect).into_response(),
            AuthResult::Redirect(target) => return Redirect::to(target.as_str()).into_response(),
            AuthResult::OK => (),
        },
        Err(e) => {
            tracing::debug!(?e, "Failed to verify token");
            return Redirect::to(redirect).into_response();
        }
    }
    next.run(request).await
}

impl<F> AuthorizedBearer<F> for Router
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_bearer(self, f: F) -> Self {
        let wrapper = move |r, n| authorize_from_bearer(r, n, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl<T, FT> AuthorizedBearerWithClaims<T, FT> for Router
where
    T: DeserializeOwned,
    FT: Send + Sync + Clone + Fn(T) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_bearer_claims(self, f: FT) -> Self {
        let f2 = move |token: &str| f(jwt::claims_for::<T>(token)?);
        let wrapper = move |r, n| authorize_from_bearer(r, n, f2.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl AuthorizedBearerWithRole for Router {
    fn authorized_bearer_role(self, role: String) -> Self {
        #[derive(Deserialize)]
        struct Claims {
            roles: Vec<String>,
        }
        self.authorized_bearer_claims(move |c: Claims| {
            if c.roles.contains(&role) {
                Ok(AuthResult::OK)
            } else {
                Ok(AuthResult::Unauthorized)
            }
        })
    }
}

impl<F> AuthorizedCookie<F> for Router
where
    F: Send + Sync + Clone + Fn(&str) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_cookie(self, redirect_to_login: &'static str, f: F) -> Self {
        let wrapper = move |r, n| authorize_from_cookie(r, n, redirect_to_login, f.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl<T, FT> AuthorizedCookieWithClaims<T, FT> for Router
where
    T: DeserializeOwned,
    FT: Send + Sync + Clone + Fn(T) -> anyhow::Result<AuthResult> + 'static,
{
    fn authorized_cookie_claims(self, redirect_to_login: &'static str, f: FT) -> Self {
        let f2 = move |token: &str| f(jwt::claims_for::<T>(token)?);
        let wrapper = move |r, n| authorize_from_cookie(r, n, redirect_to_login, f2.clone());
        self.layer(middleware::from_fn(wrapper))
    }
}

impl AuthorizedCookieWithRole for Router {
    fn authorized_cookie_role(self, redirect_to_login: &'static str, role: &'static str) -> Self {
        #[derive(Deserialize)]
        struct Claims {
            roles: Vec<String>,
        }
        self.authorized_cookie_claims(redirect_to_login, move |c: Claims| {
            if c.roles.contains(&role.to_string()) {
                Ok(AuthResult::OK)
            } else {
                Ok(AuthResult::Unauthorized)
            }
        })
    }
}

fn response_unauthorized() -> Response {
    Response::builder()
        .status(401)
        .body(Body::new("401 Unauthorized".to_string()))
        .unwrap()
}
