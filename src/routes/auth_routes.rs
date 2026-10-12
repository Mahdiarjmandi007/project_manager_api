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
use crate::auth::{self, RegisterError};

#[derive(Deserialize)]
struct RegisterRequest{
    email: String,
    password: String,
    fullname: String,
}

#[derive(Serialize)]
struct ApiMessage{
    message: String,
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


pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
}