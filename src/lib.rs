pub mod models;

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
pub async fn database_connect(db_url: &str) -> Result<SqlitePool,anyhow::Error> {

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        match Sqlite::create_database(&db_url).await {
            Ok(_) => tracing::info!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    }

    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Err connecting to database");

    if let Err(e) = sqlx::query(r#"
        CREATE TABLE
        IF NOT EXISTS
        voices_info (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel_id VARCHAR (25) NOT NULL,
            owner_id VARCHAR (25) NOT NULL
        );
    "#)
    .execute(&pool)
    .await {
        panic!("Error occurred on table creating: {}", e)
    }

    Ok(pool)
}