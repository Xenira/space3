use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// TODO: Reduce expiration time and add refresh token support
const JWT_EXPIRATION: i64 = 12 * 60_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    iat: usize,
    nbf: usize,
    pub sub: String,
}

pub fn generate(sub: &str) -> String {
    let claims = Claims {
        exp: Utc::now()
            .checked_add_signed(Duration::milliseconds(JWT_EXPIRATION))
            .expect("Expiration invalid")
            .timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        nbf: Utc::now().timestamp() as usize,
        sub: sub.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .expect("Failed to generate jwt")
}

pub fn validate(
    token: &str,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    )
}
