use axum::{
    http::{
        header::AUTHORIZATION,
        HeaderMap,
    },
};

use crate::{
    token::{decode_access_token, Claims},
    AppState,
};
pub fn authenticate(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<Claims, &'static str> {
    let authorization = headers
        .get(AUTHORIZATION)
        .ok_or("missing authorization header")?
        .to_str()
        .map_err(|_| "invalid authorization header")?;

    let access_token = authorization
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .ok_or("invalid authorization format")?;

    let claims = decode_access_token(
        access_token,
        &state.jwt_secret,
    )
    .map_err(|_| "invalid or expired access token")?;

    claims
        .sub
        .parse::<i64>()
        .map_err(|_| "invalid user id in token")?;

    Ok(claims)
}
