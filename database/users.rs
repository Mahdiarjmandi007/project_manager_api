use chrono;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub fullname: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

pub async fn find_user_by_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
    SELECT 
        id,
        email,
        password_hash,
        fullname,
        role,
        is_active,
        created_at
    FROM users
    WHERE email=?
    "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}
pub async fn find_user_by_id(pool: &SqlitePool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
    SELECT 
        id,
        email,
        password_hash,
        fullname,
        role,
        is_active,
        created_at
    FROM users
    WHERE id=?
    "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}
pub async fn get_users(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT
            id,
            email,
            password_hash,
            fullname,
            role,
            is_active,
            created_at
        FROM users
        ORDER BY id
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(users)
}
pub async fn change_user_role(
    pool: &SqlitePool,
    user_id: i64,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(role)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn change_user_status(
    pool: &SqlitePool,
    user_id: i64,
    is_active: &bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE users SET is_active = ? WHERE id = ?")
        .bind(is_active)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_user(
    pool: &SqlitePool,
    email: &str,
    password_hash: &str,
    fullname: &str,
) -> Result<(), sqlx::Error> {
    let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        r#"
        INSERT INTO users(
        email,
        password_hash,
        fullname,
        role,
        created_at
        )
        VALUES(?,?,?,'member',?)
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(fullname)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_admin(
    pool: &SqlitePool,
    email: &str,
    password_hash: &str,
    fullname: &str,
) -> Result<(), sqlx::Error> {
    let exist_user = find_user_by_email(pool, email).await?;

    if exist_user.is_some() {
        return Ok(());
    }
    let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        r#"
        INSERT INTO users(
        email,
        password_hash,
        fullname,
        role,
        created_at
        )
        VALUES(?,?,?,'admin',?)
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(fullname)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_find_user_by_email() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                fullname TEXT NOT NULL,
                role TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        create_user(&pool, "mahdi@example.com", "test_hash", "Mahdi Arjmandi")
            .await
            .unwrap();

        let user = find_user_by_email(&pool, "mahdi@example.com")
            .await
            .unwrap();

        assert!(user.is_some());

        let user = user.unwrap();

        assert_eq!(user.email, "mahdi@example.com");
        assert_eq!(user.fullname, "Mahdi Arjmandi");
        assert_eq!(user.role, "member");
    }
}
