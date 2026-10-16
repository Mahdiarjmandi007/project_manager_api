use axum::{
    Json, Router, extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post, post_service},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth_middleware::authenticate,
    database::projects::insert_project,
    AppState,
};

#[derive(Deserialize)]
struct CreateProjectRequest {
    name: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct ProjectCreatedResponse {
    message: String,
    project_id: i64,
}

#[derive(Serialize)]
struct ApiMessage {
    message: String,
}

async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let claims = match authenticate(&headers, &state) {
        Ok(claims) => claims,
        Err(message) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: message.to_string(),
                }),
            )
                .into_response();
        }
    };

    if claims.role != "admin" && claims.role != "manager" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "admin or manager permission required".to_string(),
            }),
        )
            .into_response();
    }

    let owner_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiMessage {
                    message: "invalid user id in token".to_string(),
                }),
            )
                .into_response();
        }
    };

    let name = request.name.trim();

    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "project name is required".to_string(),
            }),
        )
            .into_response();
    }

    match insert_project(
        &state.pool,
        name,
        request.description.as_deref(),
        owner_id,
    )
    .await
    {
        Ok(project_id) => (
            StatusCode::CREATED,
            Json(ProjectCreatedResponse {
                message: "project created successfully".to_string(),
                project_id,
            }),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to create project".to_string(),
            }),
        )
            .into_response(),
    }
}

// Temporary placeholder; we will implement this endpoint next.
async fn list_projects() -> &'static str {
    "Projects endpoint"
}




pub fn projects_routes() -> Router<AppState> {
    Router::new()
    .route("/", post(create_project))
}