
use sqlx::SqlitePool;

pub async fn insert_refresh_token(
    pool: &SqlitePool,
    user_id: i64,
    token_hash: &str,
    expires_at: &str,
    created_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (
            user_id,
            token_hash,
            expires_at,
            created_at
        )
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(())
}


pub async fn is_refresh_token_valid(
    pool: &SqlitePool,
    token_hash: &str,
    current_time: &str,
) -> Result<bool, sqlx::Error> {
    let is_valid: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM refresh_tokens
            WHERE token_hash = ?
              AND revoked_at IS NULL
              AND expires_at > ?
        )
        "#,
    )
    .bind(token_hash)
    .bind(current_time)
    .fetch_one(pool)
    .await?;

    Ok(is_valid)
}


pub async fn revoke_refresh_token(
    pool: &SqlitePool,
    token_hash: &str,
    current_time: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = ?
        WHERE token_hash = ?
          AND revoked_at IS NULL
        "#,
    )
    .bind(current_time)
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

