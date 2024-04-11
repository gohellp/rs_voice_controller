pub use anyhow::{Error, Result};

pub struct Data {
    pub voice_id: u64,
    pub guild_id: u64,
    pub pool: sqlx::SqlitePool
}

pub type CommandError = Error;
pub type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, CommandError>;
