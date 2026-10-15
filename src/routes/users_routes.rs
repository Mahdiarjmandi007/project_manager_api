use axum::{
    extract::{Query, State,Path},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post,put},
    Json,
    Router,
};

use serde::{Deserialize, Serialize};
use crate::{
    database::users::{find_user_by_id, get_users},
    token::decode_access_token,
    AppState,
};

#[derive(Serialize)]
struct ApiMessage{
    message: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub fullname: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct UsersResponse {
    pub users: Vec<UserResponse>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(pagination): Query<Pagination>,
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

    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "admin permission required".to_string(),
            }),
        )
            .into_response();
    }

    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    if limit < 1 || limit > 100 || offset < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "invalid pagination values".to_string(),
            }),
        )
            .into_response();
    }

    let users = match get_users(&state.pool, limit, offset).await {
        Ok(users) => users,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiMessage {
                    message: "database error".to_string(),
                }),
            )
                .into_response();
        }
    };

    let users: Vec<UserResponse> = users
        .into_iter()
        .map(|user| UserResponse {
            id: user.id,
            email: user.email,
            fullname: user.fullname,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
        })
        .collect();

    Json(users).into_response()        
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
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

    let current_user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid token subject".to_string(),
                }),
            )
                .into_response();
        }
    };

    if claims.role != "admin" && current_user_id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "you can only view your own information".to_string(),
            }),
        )
            .into_response();
    }

    let user = match find_user_by_id(&state.pool, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
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
                    message: "database error".to_string(),
                }),
            )
                .into_response();
        }
    };

    let user = UserResponse {
        id: user.id,
        email: user.email,
        fullname: user.fullname,
        role: user.role,
        is_active: user.is_active,
        created_at: user.created_at,
    };

    Json(user).into_response()
}

pub fn users_routes() -> Router<AppState> {
    Router::new()
    .route("/", get(list_users))
    .route("/{id}", get(get_user))

}