use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub assignee_id: Option<i64>,
    pub created_by: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

pub async fn create_task(
    pool: &SqlitePool,
    project_id: i64,
    title: &str,
    description: Option<&str>,
    assignee_id: Option<i64>,
    created_by: i64,
) -> Result<i64, sqlx::Error> {
    let created_at = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO tasks (
            project_id,
            title,
            description,
            status,
            assignee_id,
            created_by,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, 'todo', ?, ?, ?, ?)
        "#,
    )
    .bind(project_id)
    .bind(title)
    .bind(description)
    .bind(assignee_id)
    .bind(created_by)
    .bind(&created_at)
    .bind(&created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_task_by_id(
    pool: &SqlitePool,
    task_id: i64,
) -> Result<Option<Task>, sqlx::Error> {
    sqlx::query_as::<_, Task>(
        r#"
        SELECT
            id,
            project_id,
            title,
            description,
            status,
            assignee_id,
            created_by,
            created_at,
            updated_at
        FROM tasks
        WHERE id = ?
        "#,
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_project_tasks(
    pool: &SqlitePool,
    project_id: i64,
    status: Option<&str>,
    assignee_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Task>, sqlx::Error> {
    let mut query = String::from(
        r#"
        SELECT
            id,
            project_id,
            title,
            description,
            status,
            assignee_id,
            created_by,
            created_at,
            updated_at
        FROM tasks
        WHERE project_id = ?
        "#,
    );

    if status.is_some() {
        query.push_str(" AND status = ?");
    }

    if assignee_id.is_some() {
        query.push_str(" AND assignee_id = ?");
    }

    query.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

    let mut request = sqlx::query_as::<_, Task>(&query)
        .bind(project_id);

    if let Some(status) = status {
        request = request.bind(status);
    }

    if let Some(assignee_id) = assignee_id {
        request = request.bind(assignee_id);
    }

    request
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn update_task(
    pool: &SqlitePool,
    task_id: i64,
    title: Option<&str>,
    description: Option<&str>,
    assignee_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let updated_at = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let result = sqlx::query(
        r#"
        UPDATE tasks
        SET
            title = COALESCE(?, title),
            description = COALESCE(?, description),
            assignee_id = COALESCE(?, assignee_id),
            updated_at = ?,
            update_count = update_count + 1
        WHERE id = ?
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(assignee_id)
    .bind(&updated_at)
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}


pub async fn update_task_status(
    pool: &SqlitePool,
    task_id: i64,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let updated_at = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let result = sqlx::query(
        r#"
        UPDATE tasks
        SET status = ?,
            updated_at = ?,
            update_count = update_count + 1
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(&updated_at)
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}


pub async fn delete_task(
    pool: &SqlitePool,
    task_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM tasks WHERE id = ?"
    )
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

