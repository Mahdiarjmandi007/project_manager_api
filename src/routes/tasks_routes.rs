use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch},
};
use serde::Deserialize;

use crate::{
    AppState,
    auth_middleware::authenticate,
    database::{
        projects::{can_access_project, get_project_owner_id, is_project_member},
        tasks::{
            create_task as insert_task, delete_task, get_project_tasks, get_task_by_id,
            update_task, update_task_status,
        },
    },
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
#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    assignee_id: Option<i64>,
}

#[derive(Deserialize)]
struct UpdateTaskStatusRequest {
    status: String,
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
                message: "page must be positive and limit must be between 1 and 100".to_string(),
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

    match can_access_project(&state.pool, project_id, user_id, &claims.role).await {
        Ok(Some(true)) => {}

        Ok(Some(false)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiMessage {
                    message: "you do not have access to this project".to_string(),
                }),
            )
                .into_response();
        }

        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiMessage {
                    message: "project not found".to_string(),
                }),
            )
                .into_response();
        }

        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
            Json(TaskListResponse { tasks, page, limit }),
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

    let owner_id = match get_project_owner_id(&state.pool, project_id).await {
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

        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if claims.role != "admin" && (claims.role != "manager" || owner_id != user_id) {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to create tasks in this project".to_string(),
            }),
        )
            .into_response();
    }

    if let Some(assignee_id) = payload.assignee_id {
        let is_member = match is_project_member(&state.pool, project_id, assignee_id).await {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if !is_member {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiMessage {
                    message: "Assignee must be a member of the project".to_string(),
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

        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match can_access_project(&state.pool, task.project_id, user_id, &claims.role).await {
        Ok(Some(true)) => (StatusCode::OK, Json(task)).into_response(),

        Ok(Some(false)) => (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to access this task".to_string(),
            }),
        )
            .into_response(),

        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "Project not found".to_string(),
            }),
        )
            .into_response(),

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpdateTaskRequest>,
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

    if let Some(title) = &payload.title {
        if title.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiMessage {
                    message: "Task title cannot be empty".to_string(),
                }),
            )
                .into_response();
        }
    }

    if payload.title.is_none() && payload.description.is_none() && payload.assignee_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "At least one field must be provided".to_string(),
            }),
        )
            .into_response();
    }

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

    let allowed = if claims.role == "admin" {
        true
    } else if claims.role == "manager" {
        match get_project_owner_id(&state.pool, task.project_id).await {
            Ok(Some(owner_id)) => owner_id == user_id,
            Ok(None) => false,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        false
    };

    if !allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to edit this task".to_string(),
            }),
        )
            .into_response();
    }

    if let Some(assignee_id) = payload.assignee_id {
        let is_member = match is_project_member(&state.pool, task.project_id, assignee_id).await {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if !is_member {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiMessage {
                    message: "Assignee must be a member of the project".to_string(),
                }),
            )
                .into_response();
        }
    }

    let title = payload.title.as_deref().map(str::trim);

    match update_task(
        &state.pool,
        task_id,
        title,
        payload.description.as_deref(),
        payload.assignee_id,
    )
    .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(ApiMessage {
                message: "Task updated successfully".to_string(),
            }),
        )
            .into_response(),

        Ok(false) => StatusCode::NOT_FOUND.into_response(),

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_task_status_handler(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpdateTaskStatusRequest>,
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

    if !["todo", "in_progress", "done"].contains(&payload.status.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "Invalid task status".to_string(),
            }),
        )
            .into_response();
    }

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
    let allowed = match claims.role.as_str() {
        "admin" => true,

        "manager" => match get_project_owner_id(&state.pool, task.project_id).await {
            Ok(Some(owner_id)) => owner_id == user_id,
            Ok(None) => false,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },

        "member" => task.assignee_id == Some(user_id),

        _ => false,
    };

    if !allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to change this task's status".to_string(),
            }),
        )
            .into_response();
    }
    match update_task_status(&state.pool, task_id, &payload.status).await {
        Ok(true) => (
            StatusCode::OK,
            Json(ApiMessage {
                message: "Task status updated successfully".to_string(),
            }),
        )
            .into_response(),

        Ok(false) => StatusCode::NOT_FOUND.into_response(),

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_task_handler(
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
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let task = match get_task_by_id(&state.pool, task_id).await {
        Ok(Some(task)) => task,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let allowed = if claims.role == "admin" {
        true
    } else if claims.role == "manager" {
        match get_project_owner_id(&state.pool, task.project_id).await {
            Ok(Some(owner_id)) => owner_id == user_id,
            Ok(None) => false,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        false
    };

    if !allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiMessage {
                message: "You are not allowed to delete this task".to_string(),
            }),
        )
            .into_response();
    }

    match delete_task(&state.pool, task_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn project_tasks_routes() -> Router<AppState> {
    Router::new().route("/{id}/tasks", get(list_tasks).post(create_task_handler))
}

pub fn task_routes() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_task_handler).patch(update_task_handler))
        .route("/{id}", delete(delete_task_handler))
        .route("/{id}/status", patch(update_task_status_handler))
}
