use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

pub struct CookieToken(pub String);

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
        let value = jar.get("token").ok_or(
            axum::response::Response::builder()
                .status(401)
                .body("401 Unauthorized".to_string())
                .unwrap(),
        )?;
        Ok(Self(value.to_string()))
    }
}
