#[path = "../database/database.rs"]
mod database;

mod auth;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
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