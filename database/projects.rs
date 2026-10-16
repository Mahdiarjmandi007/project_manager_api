use sqlx::FromRow;
use sqlx::SqlitePool;

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

    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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

pub async fn update_project(
    pool: &SqlitePool,
    project_id: i64,
    name: &str,
    description: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE projects
        SET name = ?, description = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(project_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_project(pool: &SqlitePool, project_id: i64) -> Result<bool, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    // Remove project memberships first.
    sqlx::query("DELETE FROM projectmember WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    // Remove tasks belonging to this project.
    sqlx::query("DELETE FROM tasks WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    // Finally, remove the project.
    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(result.rows_affected() > 0)
}

pub async fn add_project_member(
    pool: &SqlitePool,
    project_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO projectmember (project_id, user_id)
        VALUES (?, ?)
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn remove_project_member(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM projectmember
         WHERE project_id = ? AND user_id = ?",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_project_owner_id(
    pool: &SqlitePool,
    project_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT owner_id FROM projects WHERE id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
}

pub async fn is_project_member(
    pool: &SqlitePool,
    project_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM projectmember
            WHERE project_id = ? AND user_id = ?
        )",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn can_access_project(
    pool: &SqlitePool,
    project_id: i64,
    user_id: i64,
    role: &str,
) -> Result<Option<bool>, sqlx::Error> {
    let owner_id = match get_project_owner_id(pool, project_id).await? {
        Some(id) => id,
        None => return Ok(None),
    };

    if role == "admin" {
        return Ok(Some(true));
    }

    if role == "manager" && owner_id == user_id {
        return Ok(Some(true));
    }

    if role == "manager" || role == "member" {
        let is_member = is_project_member(pool, project_id, user_id).await?;

        return Ok(Some(is_member));
    }

    Ok(Some(false))
}
