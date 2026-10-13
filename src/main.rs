
#[path = "../database/database.rs"]
mod database;
mod auth;
mod token;
mod routes;
mod seed;

use sqlx::sqlite::SqlitePoolOptions;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: String,
}

async fn create_pool() -> Result<sqlx::SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://database/project_management.db?mode=rwc")
        .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("reset-db") => {
            
            let db_path = "database/project_management.db";

            match std::fs::remove_file(db_path) {
                Ok(()) => println!("Old database deleted."),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    println!("No existing database found.")
                }
                Err(error) => return Err(error.into()),
            }

            
            let pool = create_pool().await?;
            database::create_all_tables(&pool).await?;
            pool.close().await;

            println!("Database reset completed.");
        }

        Some("seed-admin") => {
            let pool = create_pool().await?;

            database::create_all_tables(&pool).await?;
            seed::seed_admin(&pool).await?;

            pool.close().await;
        }

        Some(command) => {
            eprintln!("Unknown command: {command}");
            eprintln!("Available commands: reset-db, seed-admin");
            return Err("Unknown command".into());
        }

        None => {
            // Normal server startup.
            let jwt_secret = std::env::var("JWT_SECRET")?;
            let pool = create_pool().await?;

            database::create_all_tables(&pool).await?;

            let state = AppState {
                pool,
                jwt_secret,
            };

            let app = routes::create_router(state);

            let listener = TcpListener::bind("127.0.0.1:3000").await?;

            println!("Server is running at http://127.0.0.1:3000");

            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
