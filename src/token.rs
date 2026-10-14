use serde::{Deserialize,Serialize};
use sha2::{Digest, Sha256};
use jsonwebtoken::{
    decode,
    encode,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug,Deserialize,Serialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub token_type: String,
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
        token_type:"access".to_string(),
        exp: expiration_time as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn decode_access_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {

    let token_data=decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    let claims = token_data.claims;

    if claims.token_type != "access" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }

    Ok(claims)
}

pub fn create_refresh_token(
    user_id: i64,
    role: &str,
    secret: &str 
) ->Result<String, jsonwebtoken::errors::Error>{
    let current_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("System time is before Unix epoch")
    .as_secs();

    let expiration_time = current_time + 7 * 24 * 60 * 60;

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        token_type:"refresh".to_string(),
        exp: expiration_time as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn decode_refresh_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {

    let token_data=decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    let claims = token_data.claims;

    if claims.token_type != "refresh" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }

    Ok(claims)
}

pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());

    let result = hasher.finalize();

    format!("{:x}", result)
}


#[cfg(test)]
mod tests {

    use super::*;
    const TEST_SECRET: &str = "test-secret-for-unit-tests-only";

    #[test]
    fn test_create_and_decode_token(){
        let token=create_access_token(7, "member", TEST_SECRET,).unwrap();
        let claim=decode_access_token(&token,TEST_SECRET).unwrap();

        assert_eq!(claim.sub, "7");
        assert_eq!(claim.role, "member");

    }
    #[test]
    fn test_wrong_token(){
        let token=create_access_token(7, "member", TEST_SECRET,).unwrap();
        let wrong_token=format!("{}x", token);

        let result=decode_access_token(&wrong_token,TEST_SECRET);


        assert!(result.is_err());

    }
    #[test]
    fn test_wrong_secretkey(){
        let token=create_access_token(7, "member", TEST_SECRET,).unwrap();
        let wrong_secret=format!("{}x", TEST_SECRET);

        let result=decode_access_token(&token,&wrong_secret);


        assert!(result.is_err());

    }
}