use sqlx::SqlitePool;
pub mod users;

pub async fn create_users_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
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
    .execute(pool)
    .await?;

    Ok(())
    }

    pub async fn create_projects_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT ,
            owner_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,

            FOREIGN KEY (owner_id) REFERENCES users(id)

        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
    }


    pub async fn create_projectmember_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS projectmember (
            project_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,

            PRIMARY KEY(project_id,user_id)

            FOREIGN KEY (project_id) REFERENCES projects(id),
            FOREIGN KEY (user_id) REFERENCES users(id)

        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
    }

    pub async fn create_tasks_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            projrct_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'todo',
            assignee_id INTEGER NOT NULL,
            created_by INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            update_count INTEGER NOT NULL DEFAULT 0, 




            FOREIGN KEY (projrct_id) REFERENCES projects(id),
            FOREIGN KEY (assignee_id) REFERENCES users(id),
            FOREIGN KEY (created_by) REFERENCES users(id)

        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
    }