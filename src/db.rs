//! SQLite persistence.
//!
//! A single connection is shared behind a mutex; every call runs on tokio's blocking pool
//! so database I/O never stalls the async runtime.

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use anyhow::Context;
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::model::{Nagging, Recurrence, Reminder, User, UtcOffset};

/// Migrations are applied in order; `PRAGMA user_version` records how many have run.
/// The first one is the schema of the original .NET bot, so existing databases upgrade in place.
const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_initial.sql"),
    include_str!("../migrations/002_offsets_and_scheduling.sql"),
];

/// Timestamps are stored as UTC text in the same format the .NET bot used (`DateTime.ToString("s")`).
const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

/// Whether `upsert_user` created a new user or updated an existing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Registration {
    Created,
    Updated,
}

impl Db {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("creating database directory {}", dir.display()))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("opening database {}", path.display()))?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        Self::init(conn)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> anyhow::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> anyhow::Result<Self> {
        conn.busy_timeout(Duration::from_secs(5))?;
        migrate(&mut conn).context("migrating database schema")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    async fn call<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> rusqlite::Result<T> + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let result = tokio::task::spawn_blocking(move || {
            let mut conn = conn.lock().unwrap_or_else(PoisonError::into_inner);
            f(&mut conn)
        })
        .await
        .context("database task panicked")?;
        Ok(result?)
    }

    // ---- users -------------------------------------------------------------------------

    pub async fn upsert_user(&self, user: User) -> anyhow::Result<Registration> {
        self.call(move |conn| {
            let tx = conn.transaction()?;
            let exists = tx
                .query_row("SELECT 1 FROM users WHERE user_id = ?1", [user.id], |_| {
                    Ok(())
                })
                .optional()?
                .is_some();
            tx.execute(
                "INSERT INTO users (user_id, first_name, last_name, utc_offset_minutes)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (user_id) DO UPDATE SET
                     first_name = excluded.first_name,
                     last_name = excluded.last_name,
                     utc_offset_minutes = excluded.utc_offset_minutes",
                params![
                    user.id,
                    user.first_name,
                    user.last_name,
                    user.utc_offset.minutes()
                ],
            )?;
            tx.commit()?;
            Ok(if exists {
                Registration::Updated
            } else {
                Registration::Created
            })
        })
        .await
    }

    pub async fn user(&self, user_id: i64) -> anyhow::Result<Option<User>> {
        self.call(move |conn| {
            conn.query_row(
                "SELECT user_id, first_name, last_name, utc_offset_minutes FROM users WHERE user_id = ?1",
                [user_id],
                user_from_row,
            )
            .optional()
        })
        .await
    }

    /// Removes the user together with their reminders and naggings (via `ON DELETE CASCADE`).
    pub async fn delete_user(&self, user_id: i64) -> anyhow::Result<bool> {
        self.call(move |conn| {
            Ok(conn.execute("DELETE FROM users WHERE user_id = ?1", [user_id])? > 0)
        })
        .await
    }

    // ---- reminders ---------------------------------------------------------------------

    pub async fn add_reminder(
        &self,
        user_id: i64,
        description: String,
        due_at: DateTime<Utc>,
        recurrence: Recurrence,
    ) -> anyhow::Result<Reminder> {
        self.call(move |conn| {
            conn.execute(
                "INSERT INTO reminders (user_id, description, due_at, recurrence) VALUES (?1, ?2, ?3, ?4)",
                params![user_id, description, DbTime(due_at), recurrence.to_db()],
            )?;
            Ok(Reminder { id: conn.last_insert_rowid(), user_id, description, due_at, recurrence })
        })
        .await
    }

    pub async fn reminders_for_user(&self, user_id: i64) -> anyhow::Result<Vec<Reminder>> {
        self.call(move |conn| {
            conn.prepare(
                "SELECT reminder_id, user_id, description, due_at, recurrence
                 FROM reminders WHERE user_id = ?1 ORDER BY due_at, reminder_id",
            )?
            .query_map([user_id], reminder_from_row)?
            .collect()
        })
        .await
    }

    /// Deletes one of the user's reminders and stops any nagging it started.
    pub async fn delete_reminder(&self, user_id: i64, reminder_id: i64) -> anyhow::Result<bool> {
        self.call(move |conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "DELETE FROM naggings WHERE reminder_id = ?1 AND user_id = ?2",
                [reminder_id, user_id],
            )?;
            let removed = tx.execute(
                "DELETE FROM reminders WHERE reminder_id = ?1 AND user_id = ?2",
                [reminder_id, user_id],
            )?;
            tx.commit()?;
            Ok(removed > 0)
        })
        .await
    }

    /// Deletes all of the user's reminders and naggings; returns the number of reminders removed.
    pub async fn delete_all_reminders(&self, user_id: i64) -> anyhow::Result<usize> {
        self.call(move |conn| {
            let tx = conn.transaction()?;
            tx.execute("DELETE FROM naggings WHERE user_id = ?1", [user_id])?;
            let removed = tx.execute("DELETE FROM reminders WHERE user_id = ?1", [user_id])?;
            tx.commit()?;
            Ok(removed)
        })
        .await
    }

    pub async fn due_reminders(&self, now: DateTime<Utc>) -> anyhow::Result<Vec<Reminder>> {
        self.call(move |conn| {
            conn.prepare(
                "SELECT reminder_id, user_id, description, due_at, recurrence
                 FROM reminders WHERE due_at <= ?1 ORDER BY due_at, reminder_id",
            )?
            .query_map([DbTime(now)], reminder_from_row)?
            .collect()
        })
        .await
    }

    /// Consumes a due reminder: starts a nagging for it (replacing one left over from the
    /// previous occurrence) and either reschedules the reminder or, if it is one-off, deletes it.
    ///
    /// Returns `None` if the reminder no longer exists.
    pub async fn fire_reminder(
        &self,
        reminder_id: i64,
        now: DateTime<Utc>,
        first_nag_at: DateTime<Utc>,
    ) -> anyhow::Result<Option<Nagging>> {
        self.call(move |conn| {
            let tx = conn.transaction()?;
            let Some(reminder) = reminder_by_id(&tx, reminder_id)? else {
                return Ok(None);
            };

            tx.execute("DELETE FROM naggings WHERE reminder_id = ?1", [reminder.id])?;
            tx.execute(
                "INSERT INTO naggings (reminder_id, user_id, description, next_at) VALUES (?1, ?2, ?3, ?4)",
                params![reminder.id, reminder.user_id, reminder.description, DbTime(first_nag_at)],
            )?;
            let nagging = Nagging {
                id: tx.last_insert_rowid(),
                reminder_id: Some(reminder.id),
                user_id: reminder.user_id,
                description: reminder.description.clone(),
                next_at: first_nag_at,
            };

            match reminder.recurrence.next_after(reminder.due_at, now) {
                Some(next) => tx.execute(
                    "UPDATE reminders SET due_at = ?1 WHERE reminder_id = ?2",
                    params![DbTime(next), reminder.id],
                )?,
                None => tx.execute("DELETE FROM reminders WHERE reminder_id = ?1", [reminder.id])?,
            };

            tx.commit()?;
            Ok(Some(nagging))
        })
        .await
    }

    // ---- naggings ----------------------------------------------------------------------

    pub async fn due_naggings(&self, now: DateTime<Utc>) -> anyhow::Result<Vec<Nagging>> {
        self.call(move |conn| {
            conn.prepare(
                "SELECT nagging_id, reminder_id, user_id, description, next_at
                 FROM naggings WHERE next_at <= ?1 ORDER BY next_at, nagging_id",
            )?
            .query_map([DbTime(now)], nagging_from_row)?
            .collect()
        })
        .await
    }

    pub async fn reschedule_nagging(
        &self,
        nagging_id: i64,
        next_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.call(move |conn| {
            conn.execute(
                "UPDATE naggings SET next_at = ?1 WHERE nagging_id = ?2",
                params![DbTime(next_at), nagging_id],
            )?;
            Ok(())
        })
        .await
    }

    /// Stops a nagging. Only the user being nagged can acknowledge it.
    pub async fn acknowledge_nagging(&self, user_id: i64, nagging_id: i64) -> anyhow::Result<bool> {
        self.call(move |conn| {
            Ok(conn.execute(
                "DELETE FROM naggings WHERE nagging_id = ?1 AND user_id = ?2",
                [nagging_id, user_id],
            )? > 0)
        })
        .await
    }

    /// Stops naggings by reminder id. Handles "Acknowledge" buttons sent by the .NET bot.
    pub async fn acknowledge_reminder(
        &self,
        user_id: i64,
        reminder_id: i64,
    ) -> anyhow::Result<bool> {
        self.call(move |conn| {
            Ok(conn.execute(
                "DELETE FROM naggings WHERE reminder_id = ?1 AND user_id = ?2",
                [reminder_id, user_id],
            )? > 0)
        })
        .await
    }

    /// Stops nagging a user, e.g. after they blocked the bot.
    pub async fn delete_naggings_for_user(&self, user_id: i64) -> anyhow::Result<usize> {
        self.call(move |conn| conn.execute("DELETE FROM naggings WHERE user_id = ?1", [user_id]))
            .await
    }

    // ---- maintenance -------------------------------------------------------------------

    pub async fn vacuum(&self) -> anyhow::Result<()> {
        self.call(|conn| conn.execute_batch("VACUUM;")).await
    }
}

fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    // Table rebuilds in migrations require foreign key enforcement to be off.
    conn.pragma_update(None, "foreign_keys", false)?;
    let applied: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let applied = usize::try_from(applied).unwrap_or(0);
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        let version = i64::try_from(index + 1).expect("migration count fits in i64");
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
        tracing::info!(version, "applied database migration");
    }
    Ok(())
}

fn reminder_by_id(tx: &Transaction<'_>, reminder_id: i64) -> rusqlite::Result<Option<Reminder>> {
    tx.query_row(
        "SELECT reminder_id, user_id, description, due_at, recurrence FROM reminders WHERE reminder_id = ?1",
        [reminder_id],
        reminder_from_row,
    )
    .optional()
}

fn user_from_row(row: &Row<'_>) -> rusqlite::Result<User> {
    let minutes: i32 = row.get(3)?;
    Ok(User {
        id: row.get(0)?,
        first_name: row.get(1)?,
        last_name: row.get(2)?,
        utc_offset: UtcOffset::from_minutes(minutes)
            .ok_or_else(|| invalid_column(3, "UTC offset out of range"))?,
    })
}

fn reminder_from_row(row: &Row<'_>) -> rusqlite::Result<Reminder> {
    let recurrence: i64 = row.get(4)?;
    Ok(Reminder {
        id: row.get(0)?,
        user_id: row.get(1)?,
        description: row.get(2)?,
        due_at: row.get::<_, DbTime>(3)?.0,
        recurrence: Recurrence::from_db(recurrence)
            .ok_or_else(|| invalid_column(4, "unknown recurrence"))?,
    })
}

fn nagging_from_row(row: &Row<'_>) -> rusqlite::Result<Nagging> {
    Ok(Nagging {
        id: row.get(0)?,
        reminder_id: row.get(1)?,
        user_id: row.get(2)?,
        description: row.get(3)?,
        next_at: row.get::<_, DbTime>(4)?.0,
    })
}

fn invalid_column(index: usize, message: &'static str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Integer, message.into())
}

/// A UTC timestamp stored as `TIME_FORMAT` text. That format sorts lexically, so SQL range
/// comparisons on these columns are correct.
struct DbTime(DateTime<Utc>);

impl ToSql for DbTime {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.format(TIME_FORMAT).to_string()))
    }
}

impl FromSql for DbTime {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = value.as_str()?;
        NaiveDateTime::parse_from_str(text, TIME_FORMAT)
            .map(|naive| DbTime(naive.and_utc()))
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;
    use chrono::TimeZone;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    fn user(id: i64) -> User {
        User {
            id,
            first_name: "Ada".into(),
            last_name: String::new(),
            utc_offset: UtcOffset::from_minutes(480).unwrap(),
        }
    }

    #[tokio::test]
    async fn upgrades_a_database_created_by_the_dotnet_bot() {
        let conn = Connection::open_in_memory().unwrap();
        // The .NET bot never enabled foreign keys, so its databases can contain orphaned rows.
        conn.pragma_update(None, "foreign_keys", false).unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.execute_batch(
            "INSERT INTO users VALUES (1, 'Ada', '', 8);
             INSERT INTO reminders (user_id, description, datetime, recurring)
                 VALUES (1, 'stretch', '2026-01-01T00:00:00', 1);
             INSERT INTO reminders (user_id, description, datetime, recurring)
                 VALUES (99, 'orphan', '2026-01-01T00:00:00', 0);
             INSERT INTO naggings (reminder_id, user_id, description, datetime)
                 VALUES (1, 1, 'stretch', '2026-01-01T00:01:00');",
        )
        .unwrap();

        let db = Db::init(conn).unwrap();

        let migrated = db.user(1).await.unwrap().unwrap();
        assert_eq!(migrated.utc_offset.minutes(), 480);
        let reminders = db.reminders_for_user(1).await.unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].due_at, utc(2026, 1, 1, 0, 0));
        assert_eq!(reminders[0].recurrence, Recurrence::Daily);
        assert!(db.reminders_for_user(99).await.unwrap().is_empty());
        assert!(
            db.due_naggings(utc(2030, 1, 1, 0, 0))
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn register_twice_updates_the_user() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(
            db.upsert_user(user(1)).await.unwrap(),
            Registration::Created
        );
        let mut updated = user(1);
        updated.utc_offset = UtcOffset::from_minutes(-300).unwrap();
        assert_eq!(
            db.upsert_user(updated.clone()).await.unwrap(),
            Registration::Updated
        );
        assert_eq!(db.user(1).await.unwrap(), Some(updated));
    }

    #[tokio::test]
    async fn recurring_reminder_is_rescheduled_and_nags_until_acknowledged() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let due = utc(2026, 3, 1, 8, 0);
        let reminder = db
            .add_reminder(1, "pills".into(), due, Recurrence::Daily)
            .await
            .unwrap();

        assert!(
            db.due_reminders(due - ChronoDuration::minutes(1))
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(db.due_reminders(due).await.unwrap(), vec![reminder.clone()]);

        let nag_at = due + ChronoDuration::minutes(1);
        let nagging = db
            .fire_reminder(reminder.id, due, nag_at)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            db.reminders_for_user(1).await.unwrap()[0].due_at,
            utc(2026, 3, 2, 8, 0)
        );
        assert!(db.due_reminders(due).await.unwrap().is_empty());
        assert_eq!(
            db.due_naggings(nag_at).await.unwrap(),
            vec![nagging.clone()]
        );

        assert!(
            !db.acknowledge_nagging(2, nagging.id).await.unwrap(),
            "other users cannot acknowledge"
        );
        assert!(db.acknowledge_nagging(1, nagging.id).await.unwrap());
        assert!(db.due_naggings(nag_at).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn next_occurrence_replaces_an_unacknowledged_nagging() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let due = utc(2026, 3, 1, 8, 0);
        let reminder = db
            .add_reminder(1, "pills".into(), due, Recurrence::Daily)
            .await
            .unwrap();

        db.fire_reminder(reminder.id, due, due).await.unwrap();
        let next_day = utc(2026, 3, 2, 8, 0);
        let second = db
            .fire_reminder(reminder.id, next_day, next_day)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(db.due_naggings(next_day).await.unwrap(), vec![second]);
    }

    #[tokio::test]
    async fn one_off_reminder_is_consumed_but_keeps_nagging() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let due = utc(2026, 3, 1, 8, 0);
        let reminder = db
            .add_reminder(1, "call mum".into(), due, Recurrence::Once)
            .await
            .unwrap();

        let nagging = db
            .fire_reminder(reminder.id, due, due)
            .await
            .unwrap()
            .unwrap();
        assert!(db.reminders_for_user(1).await.unwrap().is_empty());
        let pending = db.due_naggings(due).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, nagging.id);
        assert_eq!(pending[0].reminder_id, None);
    }

    #[tokio::test]
    async fn overdue_reminder_skips_missed_occurrences() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let reminder = db
            .add_reminder(
                1,
                "water plants".into(),
                utc(2026, 3, 1, 8, 0),
                Recurrence::Daily,
            )
            .await
            .unwrap();

        let now = utc(2026, 3, 5, 12, 0);
        db.fire_reminder(reminder.id, now, now).await.unwrap();
        assert_eq!(
            db.reminders_for_user(1).await.unwrap()[0].due_at,
            utc(2026, 3, 6, 8, 0)
        );
    }

    #[tokio::test]
    async fn removing_a_reminder_stops_its_nagging() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let due = utc(2026, 3, 1, 8, 0);
        let reminder = db
            .add_reminder(1, "pills".into(), due, Recurrence::Daily)
            .await
            .unwrap();
        db.fire_reminder(reminder.id, due, due).await.unwrap();

        assert!(
            !db.delete_reminder(2, reminder.id).await.unwrap(),
            "other users cannot delete it"
        );
        assert!(db.delete_reminder(1, reminder.id).await.unwrap());
        assert!(db.due_naggings(due).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn unregistering_removes_everything() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let due = utc(2026, 3, 1, 8, 0);
        let reminder = db
            .add_reminder(1, "a".into(), due, Recurrence::Daily)
            .await
            .unwrap();
        db.add_reminder(1, "b".into(), due, Recurrence::Once)
            .await
            .unwrap();
        db.fire_reminder(reminder.id, due, due).await.unwrap();

        assert!(db.delete_user(1).await.unwrap());
        assert!(db.reminders_for_user(1).await.unwrap().is_empty());
        assert!(db.due_naggings(due).await.unwrap().is_empty());
        assert!(!db.delete_user(1).await.unwrap());
    }

    #[tokio::test]
    async fn legacy_acknowledgement_by_reminder_id() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_user(user(1)).await.unwrap();
        let due = utc(2026, 3, 1, 8, 0);
        let reminder = db
            .add_reminder(1, "pills".into(), due, Recurrence::Weekly)
            .await
            .unwrap();
        db.fire_reminder(reminder.id, due, due).await.unwrap();

        assert!(db.acknowledge_reminder(1, reminder.id).await.unwrap());
        assert!(db.due_naggings(due).await.unwrap().is_empty());
    }
}
