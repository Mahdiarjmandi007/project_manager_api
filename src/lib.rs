#[path = "../database/database.rs"]
pub mod database;

pub mod auth;
pub mod auth_middleware;
pub mod routes;
pub mod seed;
pub mod token;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: String,
}
