use argon2::{
    password_hash::{
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use sqlx::{FromRow, SqlitePool};
use rand_core::OsRng;
use crate::token::{
    create_access_token,
    create_refresh_token,
    decode_refresh_token,
    hash_refresh_token};

use crate::database::users::{
    create_user,
    find_user_by_email,
    find_user_by_id,
    User,
};

use crate::database::refresh_tokens::{
    insert_refresh_token,
    is_refresh_token_valid,
    revoke_refresh_token,
};

#[derive(Debug, FromRow)]
pub struct Tokens{
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug)]
pub enum RegisterError {
    PasswordTooShort,
    EmailAlreadyExists,
    InvalidPassword,
    EmailNotFound,
    TokenCreation,
    InvalidRefreshToken,
    InactiveUser,
    Database(sqlx::Error),
    Hash(argon2::password_hash::Error),
}

pub fn hash_password(
    password: &str,
) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn register(
    pool: &SqlitePool,
    email: &str,
    password: &str,
    fullname: &str,
) -> Result<(), RegisterError> {
    
    
    if password.chars().count() < 8 {
        return Err(RegisterError::PasswordTooShort);
    }

    
    let existing_user = find_user_by_email(pool, email)
        .await
        .map_err(RegisterError::Database)?;

    if existing_user.is_some() {
        return Err(RegisterError::EmailAlreadyExists);
    }

    let password_hash =
        hash_password(password).map_err(RegisterError::Hash)?;

    
    create_user(
        pool,
        email,
        &password_hash,
        fullname,
    )
    .await
    .map_err(RegisterError::Database)?;

    Ok(())
}

pub async fn login(
    pool: &SqlitePool,
    email: &str,
    password: &str,
    secret: &str,
) -> Result<Option<Tokens> , RegisterError> {
    let user = find_user_by_email(pool, email)
        .await
        .map_err(RegisterError::Database)?;
    if user.is_none(){
        return Err(RegisterError::EmailNotFound);
    }
    
    let user = user.unwrap();
    
    let is_valid=verify_password(password, &user.password_hash).map_err(RegisterError::Hash)?;

    if !is_valid{
        return Err(RegisterError::InvalidPassword);
    }
    let access_token = create_access_token(
        user.id,
        &user.role,
        secret,
    ).map_err(|_| RegisterError::TokenCreation)?;

    let refresh_token = create_refresh_token(
        user.id,
        &user.role,
        secret,
    ).map_err(|_| RegisterError::TokenCreation)?;
    let refresh_token_hash=hash_refresh_token(&refresh_token);

    let now = chrono::Utc::now();

    let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let expires_at = (now + chrono::Duration::days(7))
        .format("%Y-%m-%d %H:%M:%S").to_string();

    insert_refresh_token(
        pool,
        user.id,
        &refresh_token_hash,
        &expires_at,
        &created_at,
    ).await.map_err(RegisterError::Database)?;
    

    Ok(Some(Tokens {
    access_token,
    refresh_token,
}))
}

pub async fn refresh(
    pool: &SqlitePool,
    old_refresh_token: &str,
    secret: &str,
) -> Result<Tokens, RegisterError>{
    let claims=decode_refresh_token(old_refresh_token, secret).map_err(|_| RegisterError::InvalidRefreshToken)?;
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| RegisterError::InvalidRefreshToken)?;

    let old_token_hash=hash_refresh_token(old_refresh_token);

    let now = chrono::Utc::now();
    let current_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut tx = pool.begin().await.map_err(RegisterError::Database)?;
    let is_valid: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM refresh_token
            WHERE token_hash = ?
                AND user_id = ?
                AND revoked_at IS NULL
                AND expires_at > ?
        )
        "#,
    )
    .bind(&old_token_hash)
    .bind(user_id)
    .bind(&current_time)
    .fetch_one(&mut *tx)
    .await
    .map_err(RegisterError::Database)?;

    if !is_valid {
        return Err(RegisterError::InvalidRefreshToken);
    }

    let user = sqlx::query_as::<_, crate::database::users::User>(
        r#"
        SELECT id, email, password_hash, fullname, role, is_active, created_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(RegisterError::Database)?
    .ok_or(RegisterError::InvalidRefreshToken)?;

    if !user.is_active{
        return Err(RegisterError::InactiveUser)
    }

    let access_token = create_access_token(user.id, &user.role, secret).map_err(|_| RegisterError::TokenCreation)?;

    let refresh_token = create_refresh_token(user.id, &user.role, secret).map_err(|_| RegisterError::TokenCreation)?;

    let new_token_hash = hash_refresh_token(&refresh_token);
    let expires_at = (now + chrono::Duration::days(7)).format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        r#"
        UPDATE refresh_token
        SET revoked_at = ?
        WHERE token_hash = ?
            AND user_id = ?
            AND revoked_at IS NULL
    "#,
    )
    .bind(&current_time)
    .bind(&old_token_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(RegisterError::Database)?;

    if result.rows_affected() != 1 {
        return Err(RegisterError::InvalidRefreshToken);
    }

    sqlx::query(
        r#"
        INSERT INTO refresh_token (
            user_id, token_hash, expires_at, created_at
        )
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(user.id)
    .bind(&new_token_hash)
    .bind(&expires_at)
    .bind(&current_time)
    .execute(&mut *tx)
    .await
    .map_err(RegisterError::Database)?;

    tx.commit()
        .await
        .map_err(RegisterError::Database)?;



    Ok(Tokens {
        access_token,
        refresh_token,
    })
}

pub async fn logout(
    pool: &SqlitePool,
    refresh_token: &str,
    secret: &str,
) -> Result<(), RegisterError> {
    
    decode_refresh_token(refresh_token, secret).map_err(|_| RegisterError::InvalidRefreshToken)?;

    let token_hash = hash_refresh_token(refresh_token);

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        r#"
        UPDATE refresh_token
        SET revoked_at = ?
        WHERE token_hash = ?
          AND revoked_at IS NULL
          AND expires_at > ?
        "#,
    )
    .bind(&now)
    .bind(&token_hash)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(RegisterError::Database)?;

    if result.rows_affected() != 1 {
        return Err(RegisterError::InvalidRefreshToken);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn test_hash_password() {
        let password = "123456";

        let hash = hash_password(password).unwrap();

        assert!(!hash.is_empty());
    }
    #[test]
    fn test_same_password_has_different_hash() {
        let password = "123456";

        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        assert_ne!(hash1, hash2);
    }
    #[test]
    fn test_verify_correct_password() {
        let password = "123456";

        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }
    #[test]
    fn test_wrong_password() {
        let password = "123456";
        let wrong_password = "6543";

        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }
#[tokio::test]
async fn test_register_to_short_password() {
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

    create_user(
        &pool,
        "mahdi@example.com",
        "test_hash",
        "Mahdi Arjmandi",
    )
    .await
    .unwrap();

    let result = register(
        &pool,
        "mahdi@.com",
        "test",
        "Mahdi",
    )
    .await;

    assert!(matches!(
        result,
        Err(RegisterError::PasswordTooShort)
    ));
}
#[tokio::test]
async fn test_register_email_exist() {
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
    let password_hash = hash_password("correct_password").unwrap();
    create_user(
        &pool,
        "mahdi@example.com",
        &password_hash,
        "Mahdi Arjmandi",
    )
    .await
    .unwrap();
    
    let result = register(
        &pool,
        "mahdi@example.com",
        "correct_password",
        "Mahdi",
    )
    .await;

    assert!(matches!(
        result,
        Err(RegisterError::EmailAlreadyExists)
    ));
}
#[tokio::test]
async fn test_success_login(){
    let pool = SqlitePoolOptions::new()
    .connect("sqlite::memory:")
    .await
    .unwrap();

    sqlx::query(
    r#"
        CREATE TABLE refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            revoked_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )"#,
    ).execute(&pool).await.unwrap();
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
        )"#,
    ).execute(&pool).await.unwrap();

    let password_hash = hash_password("correct_password").unwrap();
    create_user(
        &pool,
        "mahdi@example.com",
        &password_hash,
        "Mahdi Arjmandi",
    )
    .await
    .unwrap();
    
    let result= login(
        &pool,
        "mahdi@example.com",
        "correct_password",
        "test").await;
    assert!(
    result.is_ok(),
    "Login failed: {:?}",result.unwrap_err());
}

#[tokio::test]
async fn test_login_email_not_found(){
    let pool = SqlitePoolOptions::new()
    .connect("sqlite::memory:")
    .await
    .unwrap();
    
    sqlx::query(
    r#"
        CREATE TABLE refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            revoked_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )"#,
    ).execute(&pool).await.unwrap();

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
        )"#,
    ).execute(&pool).await.unwrap();

    let password_hash = hash_password("correct_password").unwrap();
    create_user(
        &pool,
        "mahdi@example.com",
        &password_hash,
        "Mahdi Arjmandi",
    )
    .await
    .unwrap();
    
    let result= login(
        &pool,
        "mahdi@.com",
        "correct_password",
        "test").await;

        assert!(matches!(
        result,
        Err(RegisterError::EmailNotFound)
    ));
}

#[tokio::test]
async fn test_login_invalid_password(){
        let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
    r#"
        CREATE TABLE refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            revoked_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )"#,
    ).execute(&pool).await.unwrap();
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
        )"#,
    ).execute(&pool).await.unwrap();
    let password_hash = hash_password("correct_password").unwrap();
    create_user(
        &pool,
        "mahdi@example.com",
        &password_hash,
        "Mahdi Arjmandi",
    )
    .await
    .unwrap();
    
    let result= login(
        &pool,
        "mahdi@example.com",
        "test",
        "test").await;

        assert!(matches!(
        result,
        Err(RegisterError::InvalidPassword)
    ));
}

}
