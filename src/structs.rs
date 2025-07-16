use std::fs;

pub use anyhow::{Error, Result};

pub struct Data {
    pub voice_id: u64,
    pub guild_id: u64,
    pub pool: sqlx::SqlitePool
}

pub type CommandError = Error;
pub type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, CommandError>;


use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tokio::fs::DirBuilder;
pub async fn database_connect(db_url: &str) -> Result<SqlitePool,anyhow::Error> {
    if !fs::exists("./Database").unwrap() {
        DirBuilder::new().create("./Database").await?;
    }


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
            voice_id VARCHAR (25) NOT NULL UNIQUE,
            owner_id VARCHAR (25) NOT NULL UNIQUE
        );

        CREATE TABLE
        IF NOT EXISTS
        users (
            id VARCHAR (25) NOT NULL UNIQUE,
            return_to_owned_channel BOOLEAN DEFAULT false
        );
    "#)
    .execute(&pool)
    .await {
        panic!("Error occurred on table creating: {}", e)
    }

    Ok(pool)
}
