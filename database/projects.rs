use sqlx::SqlitePool;
use sqlx::FromRow;

#[derive(Debug, FromRow, serde::Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: i64,
    pub created_at: String,
}


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

pub async fn get_user_projects(
    pool: &SqlitePool,
    user_id: i64,
    role: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Project>, sqlx::Error> {
    let projects = if role == "admin" {
        sqlx::query_as::<_, Project>(
            r#"
            SELECT id, name, description, owner_id, created_at
            FROM projects
            ORDER BY id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Project>(
            r#"
            SELECT DISTINCT p.id, p.name, p.description, p.owner_id, p.created_at
            FROM projects p
            INNER JOIN projectmember pm ON pm.project_id = p.id
            WHERE pm.user_id = ?
            ORDER BY p.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(projects)
}


pub async fn get_project_by_id(
    pool: &SqlitePool,
    project_id: i64,
) -> Result<Option<Project>, sqlx::Error> {
    let project = sqlx::query_as::<_, Project>(
        r#"
        SELECT id, name, description, owner_id, created_at
        FROM projects
        WHERE id = ?
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;

    Ok(project)
}
