
#[path = "../database/database.rs"]
pub mod database;

pub mod auth;
pub mod token;
pub mod routes;
pub mod seed;
pub mod auth_middleware;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: String,
}
