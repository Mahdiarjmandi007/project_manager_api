use axum::{
    routing::get,
    Router,
};

use crate::AppState;

mod auth_routes;

async fn home() -> &'static str {
    "Project Manager API is running!"
}
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .nest("/auth", auth_routes::auth_routes())
        .with_state(state)
}


