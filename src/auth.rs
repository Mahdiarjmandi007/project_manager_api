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


use crate::database::users::{
    create_user,
    find_user_by_email,
};

#[derive(Debug)]
pub enum RegisterError {
    PasswordTooShort,
    EmailAlreadyExists,
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
        "mahdi@example.com",
        "test_hash",
        "Mahdi",
    )
    .await;

    assert!(matches!(
        result,
        Err(RegisterError::EmailAlreadyExists)
    ));
}
}
