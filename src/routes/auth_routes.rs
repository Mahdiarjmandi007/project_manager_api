use axum::Router;
use crate::AppState;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
}