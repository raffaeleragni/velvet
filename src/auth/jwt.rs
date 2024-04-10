use std::{collections::HashMap, env, str::FromStr};

use jsonwebtoken::{decode, decode_header, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::sync::OnceCell;

pub struct VerifiedClaims<T: DeserializeOwned>(pub Header, pub T);

pub fn claims_for<T: DeserializeOwned>(token: &str) -> anyhow::Result<T> {
    Ok(token.parse::<VerifiedClaims<T>>()?.1)
}

pub(super) static JWT_DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();
pub(super) static JWT_ENCODING_KEY: OnceCell<EncodingKey> = OnceCell::const_new();
pub(super) static JWT_DECODING_KEYS_BY_ID: OnceCell<HashMap<String, DecodingKey>> =
    OnceCell::const_new();

pub enum JWT {
    Secret,
    JwkUrl,
}

#[derive(Deserialize)]
struct JWKResponse {
    keys: Vec<jsonwebtoken::jwk::Jwk>,
}

impl JWT {
    pub async fn setup(self) -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        match self {
            JWT::Secret => {
                dotenv::dotenv().ok();
                let deckey = DecodingKey::from_secret(env::var("JWT_SECRET")?.as_ref());
                let enckey = EncodingKey::from_secret(env::var("JWT_SECRET")?.as_ref());
                JWT_DECODING_KEY.get_or_init(|| async move { deckey }).await;
                JWT_ENCODING_KEY.get_or_init(|| async move { enckey }).await;
                Ok(())
            }
            JWT::JwkUrl => {
                let url = env::var("JWK_URL")?;
                let jwk = crate::client::client()
                    .get(url)
                    .send()
                    .await?
                    .json::<JWKResponse>()
                    .await?;
                let mut keys_map = HashMap::<String, DecodingKey>::new();
                for k in jwk.keys {
                    let kid = k
                        .common
                        .key_id
                        .as_ref()
                        .ok_or(anyhow::Error::msg("no kid on jwt response"))?;
                    let dk = DecodingKey::from_jwk(&k)?;
                    tracing::debug!(kid, "key id loaded");
                    keys_map.insert(kid.to_owned(), dk);
                }
                JWT_DECODING_KEYS_BY_ID
                    .get_or_init(|| async move { keys_map })
                    .await;
                Ok(())
            }
        }
    }
}

impl<T: DeserializeOwned> FromStr for VerifiedClaims<T> {
    type Err = anyhow::Error;

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        fn get_default_key<'a>() -> Result<&'a DecodingKey, anyhow::Error> {
            JWT_DECODING_KEY
                .get()
                .ok_or(anyhow::Error::msg("DECODING_KEY was not initialized"))
        }

        let header = decode_header(token)?;
        let key = match header.kid {
            Some(kid) => {
                let map = JWT_DECODING_KEYS_BY_ID.get().ok_or(anyhow::Error::msg(
                    "JWT_DECODING_KEYS_BY_ID was not initialized",
                ))?;
                match map.get(&kid) {
                    Some(key) => key,
                    None => {
                        tracing::debug!(kid, "key id not loaded");
                        get_default_key()?
                    }
                }
            }
            None => get_default_key()?,
        };
        let decoded = decode::<T>(token, key, &Validation::default())?;
        Ok(VerifiedClaims(decoded.header, decoded.claims))
    }
}
