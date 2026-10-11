use serde::{Deserialize,Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug,Deserialize,Serialize)]
pub struct Claims{
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn create_access_token(
    user_id: i64,
    role: &str,
    secret: &str 
) ->Result<String, jsonwebtoken::errors::Error>{
    let current_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("System time is before Unix epoch")
    .as_secs();

    let expiration_time = current_time + 15 * 60;

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration_time as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}