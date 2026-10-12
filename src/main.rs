#[path = "../database/database.rs"]
mod database;
mod auth;
mod token;
mod routes;

use sqlx::sqlite::SqlitePoolOptions;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://database/project_management.db?mode=rwc")
        .await?;

    database::create_users_table(&pool).await?;

    let state = AppState {
        pool,
        jwt_secret,
    };

    let app = routes::create_router(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Server is running at http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
