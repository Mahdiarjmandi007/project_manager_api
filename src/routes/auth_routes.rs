use argon2::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::{self, RegisterError, login , Tokens};

#[derive(Deserialize)]
struct RegisterRequest{
    email: String,
    password: String,
    fullname: String,
}
#[derive(Deserialize)]
struct LoginRequest{
    email: String,
    password: String,
    
}

#[derive(Serialize)]
struct ApiMessage{
    message: String,
}
#[derive(Serialize)]
struct LoginResponse {
    message: String,
    access_token: String,
    refresh_token: String,
}

async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse{
    let result=auth::register(
        &state.pool,
        &payload.email,
        &payload.password,
        &payload.fullname,
    ).await;

    match result {
        Ok(()) =>(
            StatusCode::CREATED,
            Json(ApiMessage{
                message: "user registered successfully".to_string(),
            }),
        ),

        Err(RegisterError::PasswordTooShort) =>(
            StatusCode::BAD_REQUEST,
            Json(ApiMessage{
                message: "password must be 8 charecters".to_string(),
            }),
        ),
        Err(RegisterError::EmailAlreadyExists) => (
            StatusCode::CONFLICT,
            Json(ApiMessage {
                message: "Email already exists".to_string(),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "Internal server error".to_string(),
            }),
        ),
        
    }

}


async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse{
    let result=login(
        &state.pool, 
        &payload.email, 
        &payload.password, 
        &state.jwt_secret
    ).await;

match result {
    Ok(Some(tokens)) => (
        StatusCode::OK,
        Json(LoginResponse {
            message: "user logged in successfully".to_string(),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }),
    )
        .into_response(),

    Ok(None) => (
        StatusCode::UNAUTHORIZED,
        Json(ApiMessage {
            message: "login failed".to_string(),
        }),
    )
        .into_response(),

    Err(RegisterError::EmailNotFound) => (
        StatusCode::UNAUTHORIZED,
        Json(ApiMessage {
            message: "email not found".to_string(),
        }),
    )
        .into_response(),
    Err(RegisterError::PasswordTooShort) => (
        StatusCode::UNAUTHORIZED,
        Json(ApiMessage {
            message: "password too short".to_string(),
        }),
    )
        .into_response(),
    
    Err(RegisterError::InvalidPassword) => (
        StatusCode::UNAUTHORIZED,
        Json(ApiMessage {
            message: "invalid password".to_string(),
        }),
    )
        .into_response(),

    Err(_) => (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ApiMessage {
        message: "Internal server error".to_string(),
        }),
    )
    .into_response()

    
}


}

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
}