use sqlx::{FromRow, SqlitePool};
use chrono::{NaiveDate,Datelike,Duration};

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
)->Result<Option<User>,sqlx::Error>{
    let user=sqlx::query_as::<_,User>(
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

pub async fn create_user(
    pool: &SqlitePool,
    email: &str,
    password_hash: &str,
    fullname: &str,
)->Result<(),sqlx::Error>{

    let created_at = chrono::Local::now()
    .format("%Y-%m-%d %H:%M:%S").to_string();

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

