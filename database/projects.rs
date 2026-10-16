use sqlx::SqlitePool;

pub async fn insert_project(
    pool: &SqlitePool,
    name: &str,
    description: Option<&str>,
    owner_id: i64,
) -> Result<i64, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let created_at = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO projects (name, description, owner_id, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(owner_id)
    .bind(&created_at)
    .execute(&mut *transaction)
    .await?;

    let project_id = result.last_insert_rowid();

    sqlx::query(
        r#"
        INSERT INTO projectmember (project_id, user_id)
        VALUES (?, ?)
        "#,
    )
    .bind(project_id)
    .bind(owner_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(project_id)
}