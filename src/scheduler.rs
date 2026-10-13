//! Background loop that fires due reminders, repeats naggings and maintains the database.

use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, SubsecRound, Utc};
use tokio::time::{Instant, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use crate::db::Db;
use crate::model::Nagging;
use crate::telegram::{self, Bot};

/// How often the database is polled for due reminders and naggings.
const TICK_INTERVAL: StdDuration = StdDuration::from_secs(10);
/// How long to wait before nagging again about an unacknowledged reminder.
const NAG_INTERVAL: Duration = Duration::minutes(1);
const VACUUM_INTERVAL: StdDuration = StdDuration::from_secs(24 * 60 * 60);

pub async fn run(bot: Bot, db: Db, shutdown: CancellationToken) {
    let mut ticker = tokio::time::interval(TICK_INTERVAL);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_vacuum = Instant::now();

    loop {
        tokio::select! {
            () = shutdown.cancelled() => break,
            _ = ticker.tick() => {}
        }

        let now = Utc::now().trunc_subsecs(0);
        if let Err(error) = fire_due_reminders(&bot, &db, now).await {
            tracing::error!(?error, "failed to process due reminders");
        }
        if let Err(error) = repeat_due_naggings(&bot, &db, now).await {
            tracing::error!(?error, "failed to process due naggings");
        }

        if last_vacuum.elapsed() >= VACUUM_INTERVAL {
            match db.vacuum().await {
                Ok(()) => tracing::info!("vacuumed database"),
                Err(error) => tracing::error!(?error, "failed to vacuum database"),
            }
            last_vacuum = Instant::now();
        }
    }
    tracing::info!("scheduler stopped");
}

async fn fire_due_reminders(bot: &Bot, db: &Db, now: DateTime<Utc>) -> anyhow::Result<()> {
    for reminder in db.due_reminders(now).await? {
        // Record the firing before sending: a failed send is retried by the nagging itself.
        if let Some(nagging) = db
            .fire_reminder(reminder.id, now, now + NAG_INTERVAL)
            .await?
        {
            tracing::info!(
                reminder_id = reminder.id,
                user_id = reminder.user_id,
                "reminder fired"
            );
            deliver(bot, db, &nagging).await?;
        }
    }
    Ok(())
}

async fn repeat_due_naggings(bot: &Bot, db: &Db, now: DateTime<Utc>) -> anyhow::Result<()> {
    for nagging in db.due_naggings(now).await? {
        db.reschedule_nagging(nagging.id, now + NAG_INTERVAL)
            .await?;
        deliver(bot, db, &nagging).await?;
    }
    Ok(())
}

/// Sends one reminder message. Delivery failures are logged rather than propagated so that
/// one unreachable chat does not hold up everyone else's reminders.
async fn deliver(bot: &Bot, db: &Db, nagging: &Nagging) -> anyhow::Result<()> {
    match telegram::send_nagging(bot, nagging).await {
        Ok(true) => {}
        Ok(false) => {
            let removed = db.delete_naggings_for_user(nagging.user_id).await?;
            tracing::warn!(
                user_id = nagging.user_id,
                removed,
                "chat unreachable; stopped nagging it"
            );
        }
        Err(error) => {
            tracing::warn!(
                ?error,
                nagging_id = nagging.id,
                "failed to send reminder; will retry"
            );
        }
    }
    Ok(())
}
