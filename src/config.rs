//! Configuration from environment variables.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, bail};
use url::Url;

const DEFAULT_DATABASE_PATH: &str = "data/extremelynaggybot.db";
const DEFAULT_WEB_LISTEN: &str = "0.0.0.0:8080";

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
    /// `WEB_APP_URL`: public HTTPS address of the Mini App. The app is not offered when unset.
    pub web_app_url: Option<Url>,
    /// `WEB_LISTEN`: address the web server binds to, default `0.0.0.0:8080`.
    pub web_listen: SocketAddr,
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

        let web_app_url = var("WEB_APP_URL")
            .map(|url| {
                let url = Url::parse(&url)
                    .context("WEB_APP_URL must be a URL, e.g. https://reminders.example.com")?;
                if url.scheme() != "https" {
                    bail!("WEB_APP_URL must use https:// (Telegram requires it)");
                }
                Ok(url)
            })
            .transpose()?;
        let web_listen = var("WEB_LISTEN")
            .as_deref()
            .unwrap_or(DEFAULT_WEB_LISTEN)
            .parse()
            .context("WEB_LISTEN must be an address like 0.0.0.0:8080")?;

        Ok(Self {
            telegram_token,
            web_app_url,
            web_listen,
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
