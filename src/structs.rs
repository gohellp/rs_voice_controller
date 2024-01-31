pub use anyhow::{Error, Result};

pub struct Data {
    pub voice_id: u64,
    pub guild_id: u64
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").finish()
    }
}

pub type CommandError = Error;
#[allow(unused)]
pub type Context<'a> = poise::Context<'a, Data, CommandError>;
pub type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, CommandError>;
