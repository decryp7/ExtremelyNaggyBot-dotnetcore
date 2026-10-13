//! Domain types shared by the database, the scheduler and the command handlers.

use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use chrono::{DateTime, Duration, FixedOffset, Months, Utc};
use regex::Regex;
use thiserror::Error;

/// How often a reminder repeats once it has fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recurrence {
    Once,
    Daily,
    Weekly,
    Monthly,
}

impl Recurrence {
    /// The occurrence following `at`, or `None` for one-off reminders.
    pub fn next(self, at: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Recurrence::Once => None,
            Recurrence::Daily => Some(at + Duration::days(1)),
            Recurrence::Weekly => Some(at + Duration::weeks(1)),
            Recurrence::Monthly => at.checked_add_months(Months::new(1)),
        }
    }

    /// The first occurrence strictly after `now`, keeping the original time of day.
    pub fn next_after(self, mut at: DateTime<Utc>, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        while at <= now {
            at = self.next(at)?;
        }
        Some(at)
    }

    pub(crate) fn to_db(self) -> i64 {
        match self {
            Recurrence::Once => 0,
            Recurrence::Daily => 1,
            Recurrence::Weekly => 2,
            Recurrence::Monthly => 3,
        }
    }

    pub(crate) fn from_db(value: i64) -> Option<Self> {
        match value {
            0 => Some(Recurrence::Once),
            1 => Some(Recurrence::Daily),
            2 => Some(Recurrence::Weekly),
            3 => Some(Recurrence::Monthly),
            _ => None,
        }
    }
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Recurrence::Once => "once",
            Recurrence::Daily => "daily",
            Recurrence::Weekly => "weekly",
            Recurrence::Monthly => "monthly",
        })
    }
}

impl FromStr for Recurrence {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "once" => Ok(Recurrence::Once),
            "daily" => Ok(Recurrence::Daily),
            "weekly" => Ok(Recurrence::Weekly),
            "monthly" => Ok(Recurrence::Monthly),
            _ => Err(InputError::InvalidRecurrence),
        }
    }
}

/// A fixed offset from UTC, with minute precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcOffset {
    minutes: i32,
}

impl UtcOffset {
    const MIN_MINUTES: i32 = -12 * 60;
    const MAX_MINUTES: i32 = 14 * 60;

    pub fn from_minutes(minutes: i32) -> Option<Self> {
        (Self::MIN_MINUTES..=Self::MAX_MINUTES)
            .contains(&minutes)
            .then_some(Self { minutes })
    }

    pub fn minutes(self) -> i32 {
        self.minutes
    }

    pub fn fixed(self) -> FixedOffset {
        FixedOffset::east_opt(self.minutes * 60).expect("offset range is validated on construction")
    }
}

impl fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.minutes < 0 { '-' } else { '+' };
        let abs = self.minutes.abs();
        write!(f, "UTC{sign}{:02}:{:02}", abs / 60, abs % 60)
    }
}

impl FromStr for UtcOffset {
    type Err = InputError;

    /// Accepts `8`, `+8`, `-5`, `+5:30`, `+0530` and an optional `UTC`/`GMT` prefix.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?i)^(?:utc|gmt)?\s*([+-])?(\d{1,2})(?::?(\d{2}))?$").expect("valid regex")
        });

        let caps = PATTERN
            .captures(s.trim())
            .ok_or(InputError::InvalidOffset)?;
        let hours: i32 = caps[2].parse().map_err(|_| InputError::InvalidOffset)?;
        let minutes: i32 = caps
            .get(3)
            .map_or(Ok(0), |m| m.as_str().parse())
            .map_err(|_| InputError::InvalidOffset)?;
        if minutes >= 60 {
            return Err(InputError::InvalidOffset);
        }

        let total = hours * 60 + minutes;
        let total = if caps.get(1).is_some_and(|m| m.as_str() == "-") {
            -total
        } else {
            total
        };
        UtcOffset::from_minutes(total).ok_or(InputError::InvalidOffset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub utc_offset: UtcOffset,
}

impl User {
    pub fn local_time(&self, at: DateTime<Utc>) -> DateTime<FixedOffset> {
        at.with_timezone(&self.utc_offset.fixed())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reminder {
    pub id: i64,
    pub user_id: i64,
    pub description: String,
    pub due_at: DateTime<Utc>,
    pub recurrence: Recurrence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nagging {
    pub id: i64,
    pub reminder_id: Option<i64>,
    pub user_id: i64,
    pub description: String,
    pub next_at: DateTime<Utc>,
}

/// User-correctable input problems. The `Display` text is sent back to the user.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InputError {
    #[error(
        "That is not a valid UTC offset. Use something like +8, -5 or +5:30 (between -12:00 and +14:00)."
    )]
    InvalidOffset,
    #[error("Recurrence must be one of: once, daily, weekly, monthly.")]
    InvalidRecurrence,
    #[error("That date/time does not exist. Use dd/mm/yyyy HH:MM in 24-hour format.")]
    InvalidDateTime,
    #[error("That time is already in the past.")]
    InPast,
    #[error("The description cannot be empty.")]
    EmptyDescription,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    #[test]
    fn parses_offsets() {
        for (input, minutes) in [
            ("8", 480),
            ("+8", 480),
            ("-5", -300),
            ("+5:30", 330),
            ("+0545", 345),
            ("UTC-03:30", -210),
            ("0", 0),
            ("+14", 840),
            ("-12", -720),
        ] {
            assert_eq!(
                input.parse::<UtcOffset>().unwrap().minutes(),
                minutes,
                "{input}"
            );
        }
        for input in ["", "abc", "+15", "-13", "+5:60", "8.5"] {
            assert!(input.parse::<UtcOffset>().is_err(), "{input}");
        }
    }

    #[test]
    fn displays_offsets() {
        assert_eq!(
            UtcOffset::from_minutes(330).unwrap().to_string(),
            "UTC+05:30"
        );
        assert_eq!(
            UtcOffset::from_minutes(-210).unwrap().to_string(),
            "UTC-03:30"
        );
        assert_eq!(UtcOffset::from_minutes(0).unwrap().to_string(), "UTC+00:00");
    }

    #[test]
    fn next_after_keeps_time_of_day() {
        let due = utc(2026, 1, 1, 8, 0);
        let now = utc(2026, 1, 10, 9, 30);
        assert_eq!(
            Recurrence::Daily.next_after(due, now),
            Some(utc(2026, 1, 11, 8, 0))
        );
        assert_eq!(
            Recurrence::Weekly.next_after(due, now),
            Some(utc(2026, 1, 15, 8, 0))
        );
        assert_eq!(
            Recurrence::Monthly.next_after(due, now),
            Some(utc(2026, 2, 1, 8, 0))
        );
        assert_eq!(Recurrence::Once.next_after(due, now), None);
    }

    #[test]
    fn next_after_is_strictly_later() {
        let due = utc(2026, 1, 1, 8, 0);
        assert_eq!(
            Recurrence::Daily.next_after(due, due),
            Some(utc(2026, 1, 2, 8, 0))
        );
    }

    #[test]
    fn recurrence_round_trips_through_db_value() {
        for r in [
            Recurrence::Once,
            Recurrence::Daily,
            Recurrence::Weekly,
            Recurrence::Monthly,
        ] {
            assert_eq!(Recurrence::from_db(r.to_db()), Some(r));
            assert_eq!(r.to_string().parse::<Recurrence>().unwrap(), r);
        }
    }
}
