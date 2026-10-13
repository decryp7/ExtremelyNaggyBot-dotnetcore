//! Configuration from environment variables.

use std::env;
use std::path::PathBuf;

use anyhow::{Context, bail};

const DEFAULT_DATABASE_PATH: &str = "data/extremelynaggybot.db";

#[derive(Debug)]
pub struct Config {
    /// `TELEGRAM_BOT_TOKEN` (required).
    pub telegram_token: String,
    /// `ADMIN_CHATID`: chat that receives an "online" message at startup.
    pub admin_chat_id: Option<i64>,
    /// `SENTRY_DSN`: error reporting is disabled when unset.
    pub sentry_dsn: Option<String>,
    /// `DATABASE_PATH`, default `data/extremelynaggybot.db`.
    pub database_path: PathBuf,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let Some(telegram_token) = var("TELEGRAM_BOT_TOKEN") else {
            bail!("TELEGRAM_BOT_TOKEN environment variable is required");
        };
        let admin_chat_id = var("ADMIN_CHATID")
            .map(|id| {
                id.parse()
                    .context("ADMIN_CHATID must be an integer chat id")
            })
            .transpose()?;

        Ok(Self {
            telegram_token,
            admin_chat_id,
            sentry_dsn: var("SENTRY_DSN"),
            database_path: var("DATABASE_PATH")
                .map_or_else(|| DEFAULT_DATABASE_PATH.into(), PathBuf::from),
        })
    }
}

/// Reads an environment variable, treating empty values as unset.
fn var(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}
