use axum::{Router, routing::get};

use crate::AppState;
mod auth_routes;
mod projects_routes;
mod tasks_routes;
mod users_routes;

async fn home() -> &'static str {
    "Project Manager API is running!"
}
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .nest("/auth", auth_routes::auth_routes())
        .nest("/users", users_routes::users_routes())
        .nest(
            "/projects",
            projects_routes::projects_routes().merge(tasks_routes::project_tasks_routes()),
        )
        .nest("/tasks", tasks_routes::task_routes())
        .with_state(state)
}
