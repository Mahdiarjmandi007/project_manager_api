#[path = "../database/database.rs"]
pub mod database;
mod auth;
mod token;
use dotenvy;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");
    println!("loaded token successfully");



    println!("Current directory: {:?}", std::env::current_dir());
    let pool = sqlx::SqlitePool::connect(
        "sqlite://database/project_management.db?mode=rwc"
    )
    .await?;

    database::create_users_table(&pool).await?;
    database::create_projects_table(&pool).await?;
    database::create_projectmember_table(&pool).await?;
    database::create_tasks_table(&pool).await?;

    println!("Database initialized successfully.");

    Ok(())
}