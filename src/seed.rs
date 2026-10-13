
use sqlx::SqlitePool;

use crate::auth::hash_password;
use crate::database::users::create_admin;

pub async fn seed_admin(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let email = std::env::var("SEED_ADMIN_EMAIL")?;
    let password = std::env::var("SEED_ADMIN_PASSWORD")?;
    let fullname = std::env::var("SEED_ADMIN_FULLNAME")?;

    let password_hash = hash_password(&password)
    .map_err(|e| format!("Failed to hash admin password: {e}"))?;

    create_admin(
        pool,
        &email,
        &password_hash,
        &fullname,
    )
    .await?;

    println!("Admin seed completed.");

    Ok(())
}
