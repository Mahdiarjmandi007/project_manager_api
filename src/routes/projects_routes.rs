use axum::{
    Json,
    Router, 
    extract::{State,Query,Path}, 
    http::{HeaderMap, StatusCode}, 
    response::IntoResponse, 
    routing::{get, post,},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth_middleware::authenticate,
    database::projects::{
        insert_project,
        get_user_projects,
        get_project_by_id
    },
    AppState,
};

#[derive(Deserialize)]
struct ProjectListQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct ProjectListResponse {
    projects: Vec<crate::database::projects::Project>,
    page: i64,
    limit: i64,
}

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


async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProjectListQuery>,
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

    let user_id = match claims.sub.parse::<i64>() {
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

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);

    if page < 1 || limit < 1 || limit > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "page must be positive and limit must be between 1 and 100"
                    .to_string(),
            }),
        )
            .into_response();
    }

    let offset = match (page - 1).checked_mul(limit) {
        Some(offset) => offset,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiMessage {
                    message: "invalid pagination values".to_string(),
                }),
            )
                .into_response();
        }
    };

    match get_user_projects(
        &state.pool,
        user_id,
        &claims.role,
        limit,
        offset,
    )
    .await
    {
        Ok(projects) => (
            StatusCode::OK,
            Json(ProjectListResponse {
                projects,
                page,
                limit,
            }),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to fetch projects".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: HeaderMap,
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

    let user_id = match claims.sub.parse::<i64>() {
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

    let project = match get_project_by_id(&state.pool, project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiMessage {
                    message: "project not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiMessage {
                    message: "failed to fetch project".to_string(),
                }),
            )
                .into_response();
        }
    };

    if claims.role != "admin" {
        let is_member = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM projectmember
                WHERE project_id = ? AND user_id = ?
            )
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await;

        match is_member {
            Ok(1) => {}
            Ok(_) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiMessage {
                        message: "you do not have access to this project".to_string(),
                    }),
                )
                    .into_response();
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiMessage {
                        message: "failed to check project access".to_string(),
                    }),
                )
                    .into_response();
            }
        }
    }

    (StatusCode::OK, Json(project)).into_response()
}



pub fn projects_routes() -> Router<AppState> {
    Router::new()
    .route("/", post(create_project))
    .route("/", get(list_projects))
    .route("/{id}", get(get_project))
}