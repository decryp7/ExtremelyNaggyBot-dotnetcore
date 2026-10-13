//! Inline button actions and their compact encoding in Telegram callback data (max 64 bytes).
//!
//! Each button carries everything needed to render the next step (the date or time picked so
//! far), so the flow does not keep per-chat state on the server beyond the draft text.

use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::model::{Recurrence, Snooze};

/// What a time picker is for: a new reminder (draft) or an existing reminder being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    Draft(i64),
    Reminder(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quick choices: "in 15 min", "tomorrow 09:00", "pick a date".
    When(Subject),
    /// Month calendar.
    Calendar(Subject, i32, u32),
    /// A day was picked; choose the hour.
    Day(Subject, NaiveDate),
    /// An hour was picked; choose the minute.
    Hour(Subject, NaiveDate, u32),
    /// A time was picked; choose how often.
    At(Subject, NaiveDateTime),
    Save(Subject, NaiveDateTime, Recurrence),
    CancelDraft(i64),
    Manage(i64),
    Delete(i64),
    List,
    Done(i64),
    Snooze(i64, Snooze),
    /// `ack:<nagging_id>` from the first Rust release.
    LegacyNaggingAck(i64),
    /// `{"Id": <reminder_id>}` from the .NET bot.
    LegacyReminderAck(i64),
    /// Replace the calendar feed link, revoking the old one.
    CalendarReset,
    /// Calendar padding and headers.
    Noop,
}

const DATE: &str = "%Y%m%d";
const DATE_TIME: &str = "%Y%m%d%H%M";

impl Action {
    pub fn encode(self) -> String {
        match self {
            Action::When(s) => format!("w:{}", subject(s)),
            Action::Calendar(s, year, month) => format!("c:{}:{year:04}{month:02}", subject(s)),
            Action::Day(s, date) => format!("D:{}:{}", subject(s), date.format(DATE)),
            Action::Hour(s, date, hour) => {
                format!("H:{}:{}{hour:02}", subject(s), date.format(DATE))
            }
            Action::At(s, at) => format!("A:{}:{}", subject(s), at.format(DATE_TIME)),
            Action::Save(s, at, r) => {
                format!("S:{}:{}:{}", subject(s), at.format(DATE_TIME), r.to_db())
            }
            Action::CancelDraft(id) => format!("x:{id}"),
            Action::Manage(id) => format!("m:{id}"),
            Action::Delete(id) => format!("del:{id}"),
            Action::List => "list".into(),
            Action::Done(id) => format!("n:{id}:ok"),
            Action::Snooze(id, snooze) => format!(
                "n:{id}:{}",
                match snooze {
                    Snooze::TenMinutes => "10m",
                    Snooze::OneHour => "1h",
                    Snooze::TomorrowMorning => "tm",
                }
            ),
            Action::LegacyNaggingAck(id) => format!("ack:{id}"),
            Action::LegacyReminderAck(id) => format!(r#"{{"Id":{id}}}"#),
            Action::CalendarReset => "cal:reset".into(),
            Action::Noop => "-".into(),
        }
    }

    pub fn parse(data: &str) -> Option<Self> {
        if data.starts_with('{') {
            #[derive(Deserialize)]
            struct Legacy {
                #[serde(rename = "Id")]
                id: i64,
            }
            return serde_json::from_str::<Legacy>(data)
                .ok()
                .map(|legacy| Action::LegacyReminderAck(legacy.id));
        }

        let parts: Vec<&str> = data.split(':').collect();
        Some(match parts.as_slice() {
            ["-"] => Action::Noop,
            ["list"] => Action::List,
            ["cal", "reset"] => Action::CalendarReset,
            ["w", s] => Action::When(parse_subject(s)?),
            ["c", s, ym] if ym.len() == 6 => {
                let month: u32 = ym[4..].parse().ok()?;
                NaiveDate::from_ymd_opt(ym[..4].parse().ok()?, month, 1)?;
                Action::Calendar(parse_subject(s)?, ym[..4].parse().ok()?, month)
            }
            ["D", s, date] => Action::Day(
                parse_subject(s)?,
                NaiveDate::parse_from_str(date, DATE).ok()?,
            ),
            ["H", s, dh] if dh.len() == 10 => {
                let hour: u32 = dh[8..].parse().ok().filter(|h| *h < 24)?;
                Action::Hour(
                    parse_subject(s)?,
                    NaiveDate::parse_from_str(&dh[..8], DATE).ok()?,
                    hour,
                )
            }
            ["A", s, at] => Action::At(
                parse_subject(s)?,
                NaiveDateTime::parse_from_str(at, DATE_TIME).ok()?,
            ),
            ["S", s, at, r] => Action::Save(
                parse_subject(s)?,
                NaiveDateTime::parse_from_str(at, DATE_TIME).ok()?,
                Recurrence::from_db(r.parse().ok()?)?,
            ),
            ["x", id] => Action::CancelDraft(id.parse().ok()?),
            ["m", id] => Action::Manage(id.parse().ok()?),
            ["del", id] => Action::Delete(id.parse().ok()?),
            ["n", id, "ok"] => Action::Done(id.parse().ok()?),
            ["n", id, "10m"] => Action::Snooze(id.parse().ok()?, Snooze::TenMinutes),
            ["n", id, "1h"] => Action::Snooze(id.parse().ok()?, Snooze::OneHour),
            ["n", id, "tm"] => Action::Snooze(id.parse().ok()?, Snooze::TomorrowMorning),
            ["ack", id] => Action::LegacyNaggingAck(id.parse().ok()?),
            _ => return None,
        })
    }
}

fn subject(s: Subject) -> String {
    match s {
        Subject::Draft(id) => format!("d{id}"),
        Subject::Reminder(id) => format!("r{id}"),
    }
}

fn parse_subject(s: &str) -> Option<Subject> {
    let (kind, id) = s.split_at_checked(1)?;
    let id = id.parse().ok()?;
    match kind {
        "d" => Some(Subject::Draft(id)),
        "r" => Some(Subject::Reminder(id)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_fits_in_64_bytes() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let at = date.and_hms_opt(23, 55, 0).unwrap();
        let big = i64::MAX;
        for action in [
            Action::When(Subject::Draft(big)),
            Action::Calendar(Subject::Reminder(big), 2026, 12),
            Action::Day(Subject::Draft(big), date),
            Action::Hour(Subject::Draft(big), date, 23),
            Action::At(Subject::Reminder(big), at),
            Action::Save(Subject::Draft(big), at, Recurrence::Yearly),
            Action::CancelDraft(big),
            Action::Manage(big),
            Action::Delete(big),
            Action::List,
            Action::Done(big),
            Action::Snooze(big, Snooze::TenMinutes),
            Action::Snooze(big, Snooze::OneHour),
            Action::Snooze(big, Snooze::TomorrowMorning),
            Action::LegacyNaggingAck(big),
            Action::LegacyReminderAck(big),
            Action::CalendarReset,
            Action::Noop,
        ] {
            let data = action.encode();
            assert!(data.len() <= 64, "{data} is {} bytes", data.len());
            assert_eq!(Action::parse(&data), Some(action), "{data}");
        }
    }

    #[test]
    fn rejects_garbage() {
        for data in [
            "",
            "w:x1",
            "c:d1:202613",
            "H:d1:2026010124",
            "S:d1:202601010900:9",
            "n:1:zz",
            "{}",
        ] {
            assert_eq!(Action::parse(data), None, "{data}");
        }
    }
}
