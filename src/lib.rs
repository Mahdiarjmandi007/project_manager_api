
#[path = "../database/database.rs"]
pub mod database;

pub mod auth;
pub mod token;
pub mod routes;
pub mod seed;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: String,
}
