
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post,put},
    Json,
    Router,
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::{self, RegisterError, login ,refresh,logout};
use crate::token::decode_access_token;
use crate::database::users::find_user_by_id;

//struct for request
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

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}
#[derive(Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
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
#[derive(Serialize)]
struct refreshResponse {
    message: String,
    access_token: String,
    refresh_token: String,
}
#[derive(Serialize)]
struct MeResponse {
    id: i64,
    email: String,
    fullname: String,
    role: String,
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
        .into_response(),

    
}

}
async fn refresh_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse{
    let result=refresh(&state.pool, &payload.refresh_token, &state.jwt_secret).await;
    match result {
    Ok(tokens) => (
        StatusCode::OK,
        Json(LoginResponse {
            message: "user logged in successfully".to_string(),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }),
    )
        .into_response(),
    Err(RegisterError::InvalidRefreshToken) => (
        StatusCode::UNAUTHORIZED,
        Json(ApiMessage {
            message: "invalid refresh token".to_string(),
        }),
    )
        .into_response(),

    Err(RegisterError::InactiveUser) => (
        StatusCode::UNAUTHORIZED,
        Json(ApiMessage {
            message: "user not is active".to_string(),
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

async fn logout_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse{
    let result=logout(&state.pool, &payload.refresh_token, &state.jwt_secret).await;

    match result {
        Ok(()) =>(
            StatusCode::OK,
            Json(ApiMessage {
                message: "user logged out successfully".to_string(),
        }),
        )
            .into_response(),

        Err(RegisterError::InvalidRefreshToken) => (
            StatusCode::UNAUTHORIZED,
            Json(ApiMessage {
                message: "invalid refresh token".to_string(),
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


async fn me_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    
    let authorization = match headers.get("Authorization") {
        Some(value) => value,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "missing authorization header".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    let authorization = match authorization.to_str() {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid authorization header".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    let access_token = match authorization.strip_prefix("Bearer ") {
        Some(token) if !token.is_empty() => token,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid authorization format".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    let claims = match decode_access_token(
        access_token,
        &state.jwt_secret,
    ) {
        Ok(claims) => claims,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid or expired access token".to_string(),
                }),
            )
                .into_response();
        }
    };

   
    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid access token".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    let user = match find_user_by_id(&state.pool, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "user not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiMessage {
                    message: "Internal server error".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "user account is inactive".to_string(),
            }),
        )
            .into_response();
    }

   
    (
        StatusCode::OK,
        Json(MeResponse {
            id: user.id,
            email: user.email,
            fullname: user.fullname,
            role: user.role,
        }),
    )
        .into_response()
}

async fn change_password_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse{
    let authorization = match headers.get("Authorization") {
        Some(value) => value,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "missing authorization header".to_string(),
                }),
            )
                .into_response();
        }
    };

    let authorization = match authorization.to_str() {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid authorization header".to_string(),
                }),
            )
                .into_response();
        }
    };
    let access_token = match authorization.strip_prefix("Bearer ") {
        Some(token) if !token.is_empty() => token,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid authorization format".to_string(),
                }),
            )
                .into_response();
        }
    };
    let claims = match decode_access_token(
        access_token,
        &state.jwt_secret,
    ) {
        Ok(claims) => claims,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid or expired access token".to_string(),
                }),
            )
                .into_response();
        }
    };
    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid access token".to_string(),
                }),
            )
                .into_response();
        }
    };
    match crate::auth::change_password(
        &state.pool,
        user_id,
        &payload.current_password,
        &payload.new_password,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiMessage {
                message: "password changed successfully".to_string(),
            }),
        )
            .into_response(),

        Err(RegisterError::InvalidPassword) => (
            StatusCode::UNAUTHORIZED,
            Json(ApiMessage {
                message: "current password is incorrect".to_string(),
            }),
        )
            .into_response(),

        Err(RegisterError::PasswordTooShort) => (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "new password must be at least 8 bytes long".to_string(),
            }),
        )
            .into_response(),

        Err(RegisterError::InactiveUser) => (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "user account is inactive".to_string(),
            }),
        )
            .into_response(),

        Err(RegisterError::UserNotFound) => (
            StatusCode::UNAUTHORIZED,
            Json(ApiMessage {
                message: "user not found".to_string(),
            }),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "Internal server error".to_string(),
            }),
        )
            .into_response(),
    }


}


pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
        .route("/logout", post(logout_handler))
        .route("/me", get(me_handler))
        .route("/me/password", put(change_password_handler))
}