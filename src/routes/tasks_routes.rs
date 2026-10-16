
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use sqlx::query_scalar;
use crate::{
    auth_middleware::authenticate,
    database::tasks::{
    create_task as insert_task,
    get_project_tasks,
    get_task_by_id,
    },
    AppState,
};

#[derive(Deserialize)]
struct TaskQuery {
    page: Option<i64>,
    limit: Option<i64>,
    status: Option<String>,
    assignee_id: Option<i64>,
}

#[derive(serde::Serialize)]
struct ApiMessage {
    message: String,
}

#[derive(serde::Serialize)]
struct TaskListResponse {
    tasks: Vec<crate::database::tasks::Task>,
    page: i64,
    limit: i64,
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: Option<String>,
    assignee_id: Option<i64>,
}

#[derive(serde::Serialize)]
struct TaskCreatedResponse {
    message: String,
    task_id: i64,
}


async fn list_tasks(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Query(query): Query<TaskQuery>,
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
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
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

    if let Some(status) = query.status.as_deref() {
        if !["todo", "in_progress", "done"].contains(&status) {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiMessage {
                    message: "invalid task status".to_string(),
                }),
            )
                .into_response();
        }
    }

    let offset = match (page - 1).checked_mul(limit) {
        Some(value) => value,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    
    let project_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)",
    )
    .bind(project_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(exists) => exists,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if !project_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "project not found".to_string(),
            }),
        )
            .into_response();
    }

    if claims.role != "admin" {
        let is_member = match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM projectmember
                WHERE project_id = ? AND user_id = ?
            )",
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
        {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if !is_member {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiMessage {
                    message: "you do not have access to this project".to_string(),
                }),
            )
                .into_response();
        }
    }

    match get_project_tasks(
        &state.pool,
        project_id,
        query.status.as_deref(),
        query.assignee_id,
        limit,
        offset,
    )
    .await
    {
        Ok(tasks) => (
            StatusCode::OK,
            Json(TaskListResponse {
                tasks,
                page,
                limit,
            }),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_task_handler(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<CreateTaskRequest>,
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
                    message: "Invalid user ID in token".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    let title = payload.title.trim();

    if title.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "Task title cannot be empty".to_string(),
            }),
        )
            .into_response();
    }

    
    let owner_id = match query_scalar::<_, i64>(
        "SELECT owner_id FROM projects WHERE id = ?",
    )
    .bind(project_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(owner_id)) => owner_id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiMessage {
                    message: "Project not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    
    if claims.role != "admin"
        && (claims.role != "manager" || owner_id != user_id)
    {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to create tasks in this project"
                    .to_string(),
            }),
        )
            .into_response();
    }

    
    if let Some(assignee_id) = payload.assignee_id {
        let is_member = match query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM projectmember
                WHERE project_id = ? AND user_id = ?
            )",
        )
        .bind(project_id)
        .bind(assignee_id)
        .fetch_one(&state.pool)
        .await
        {
            Ok(value) => value,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        if !is_member {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiMessage {
                    message: "Assignee must be a member of the project"
                        .to_string(),
                }),
            )
                .into_response();
        }
    }

    
    match insert_task(
        &state.pool,
        project_id,
        title,
        payload.description.as_deref(),
        payload.assignee_id,
        user_id,
    )
    .await
    {
        Ok(task_id) => (
            StatusCode::CREATED,
            Json(TaskCreatedResponse {
                message: "Task created successfully".to_string(),
                task_id,
            }),
        )
            .into_response(),

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
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
                    message: "Invalid user ID in token".to_string(),
                }),
            )
                .into_response();
        }
    };

    
    let task = match get_task_by_id(&state.pool, task_id).await {
        Ok(Some(task)) => task,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiMessage {
                    message: "Task not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    
    if claims.role == "admin" {
        return (StatusCode::OK, Json(task)).into_response();
    }

    
    let has_access = if claims.role == "manager" {
        query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM projects
                WHERE id = ? AND owner_id = ?
            )",
        )
        .bind(task.project_id)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
    } else {
        query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM projectmember
                WHERE project_id = ? AND user_id = ?
            )",
        )
        .bind(task.project_id)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
    };

    match has_access {
        Ok(true) => (StatusCode::OK, Json(task)).into_response(),
        Ok(false) => (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to access this task".to_string(),
            }),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn project_tasks_routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/tasks", get(list_tasks).post(create_task_handler))
}

pub fn task_routes() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_task_handler))
}
