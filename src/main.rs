mod commands;
mod config;
mod db;
mod model;
mod scheduler;
mod telegram;

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Context;
use teloxide::adaptors::throttle::Limits;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::commands::{BotInfo, Command};
use crate::config::Config;
use crate::db::Db;
use crate::telegram::AppState;

fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Configuration error: {error:#}");
            return ExitCode::FAILURE;
        }
    };

    // Sentry must be initialised before the async runtime starts.
    let _sentry = config.sentry_dsn.as_deref().map(|dsn| {
        let mut options = sentry::ClientOptions::default();
        options.release = sentry::release_name!();
        sentry::init((dsn, options))
    });

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(?error, "failed to start async runtime");
            return ExitCode::FAILURE;
        }
    };

    match runtime.block_on(run(config)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = format!("{error:#}"), "fatal error");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: Config) -> anyhow::Result<()> {
    let db = Db::open(&config.database_path)?;
    tracing::info!(path = %config.database_path.display(), "database ready");

    let bot = teloxide::Bot::new(&config.telegram_token).throttle(Limits::default());
    let me = bot.get_me().await.context("connecting to Telegram")?;
    let bot_info = BotInfo {
        name: me.first_name.clone(),
        username: me.username().to_owned(),
    };
    tracing::info!(id = %me.id, username = %bot_info.username, "connected to Telegram");

    if let Err(error) = bot.set_my_commands(Command::bot_commands()).await {
        tracing::warn!(?error, "failed to publish the command list");
    }
    if let Some(chat_id) = config.admin_chat_id
        && let Err(error) = bot
            .send_message(ChatId(chat_id), "Extremely Naggy Bot is online!")
            .await
    {
        tracing::warn!(?error, "failed to notify admin chat");
    }

    let shutdown = CancellationToken::new();
    let scheduler = tokio::spawn(scheduler::run(bot.clone(), db.clone(), shutdown.clone()));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(telegram::handle_message))
        .branch(Update::filter_callback_query().endpoint(telegram::handle_callback));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(AppState { db, bot_info })])
        .default_handler(|_| async {})
        .error_handler(LoggingErrorHandler::with_custom_text(
            "error while handling an update",
        ))
        .build();

    let dispatcher_token = dispatcher.shutdown_token();
    let signal_shutdown = shutdown.clone();
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        tracing::info!("shutting down");
        signal_shutdown.cancel();
        if let Ok(done) = dispatcher_token.shutdown() {
            done.await;
        }
    });

    dispatcher.dispatch().await;
    shutdown.cancel();
    scheduler.await.context("scheduler task panicked")?;
    Ok(())
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM (what `docker stop` sends).
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(?error, "failed to listen for Ctrl+C");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(?error, "failed to listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
