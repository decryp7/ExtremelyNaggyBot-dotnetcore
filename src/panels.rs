//! Message text and inline keyboards for each step of the reminder flow.

use chrono::{
    DateTime, Datelike, Duration, Months, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc,
};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::callback::{Action, Snooze, Subject};
use crate::model::{Recurrence, Reminder, User};

pub struct Panel {
    pub text: String,
    pub keyboard: InlineKeyboardMarkup,
}

/// How far ahead the calendar can be paged.
const MAX_MONTHS_AHEAD: u32 = 60;
/// Most reminders shown (with buttons) by `/list`; Telegram limits keyboard size.
const MAX_LISTED: usize = 30;

pub const HINT_TYPE_TIME: &str =
    "Tap a button, or type a time like \"fri 6pm\", \"25/12 9:00\" or \"in 2 hours\".";

fn button(label: impl Into<String>, action: Action) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(label, action.encode())
}

fn blank() -> InlineKeyboardButton {
    button(" ", Action::Noop)
}

pub fn format_when(at: NaiveDateTime) -> String {
    at.format("%a %-d %b %Y, %H:%M").to_string()
}

/// "in 5 min", "in 3 h 20 min", "in 4 days".
pub fn relative(from: DateTime<Utc>, to: DateTime<Utc>) -> String {
    let minutes = (to - from).num_minutes().max(0);
    match minutes {
        0 => "now".into(),
        1..=59 => format!("in {minutes} min"),
        60..=2879 => match minutes % 60 {
            0 => format!("in {} h", minutes / 60),
            m => format!("in {} h {m} min", minutes / 60),
        },
        _ => format!("in {} days", minutes / 1440),
    }
}

fn back_or_cancel(subject: Subject, back: Action) -> Vec<InlineKeyboardButton> {
    let mut row = vec![button("⬅ Back", back)];
    if let Subject::Draft(id) = subject {
        row.push(button("✖ Cancel", Action::CancelDraft(id)));
    }
    row
}

/// Step 1: quick choices.
pub fn when(subject: Subject, text: &str, now: NaiveDateTime) -> Panel {
    let in_minutes = |m: i64| truncate_to_minute(now + Duration::minutes(m));
    let today = now.date();
    let evening = today.and_time(time(20, 0));
    let (evening_label, evening) = if now < evening - Duration::minutes(30) {
        ("This evening 20:00", evening)
    } else {
        ("Tomorrow 20:00", evening + Duration::days(1))
    };
    let tomorrow_morning = (today + Duration::days(1)).and_time(time(9, 0));

    let mut rows = vec![
        vec![
            button("15 min", Action::At(subject, in_minutes(15))),
            button("1 hour", Action::At(subject, in_minutes(60))),
            button("3 hours", Action::At(subject, in_minutes(180))),
        ],
        vec![
            button(evening_label, Action::At(subject, evening)),
            button("Tomorrow 09:00", Action::At(subject, tomorrow_morning)),
        ],
        vec![button(
            "📅 Pick a date",
            Action::Calendar(subject, today.year(), today.month()),
        )],
    ];
    let prompt = match subject {
        Subject::Draft(id) => {
            rows.push(vec![button("✖ Cancel", Action::CancelDraft(id))]);
            format!("When should I remind you?\n{HINT_TYPE_TIME}")
        }
        Subject::Reminder(id) => {
            rows.push(vec![button("⬅ Back", Action::Manage(id))]);
            "Pick a new time:".to_owned()
        }
    };
    Panel {
        text: format!("📝 {text}\n\n{prompt}"),
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

/// Step 2: month calendar. Past days are not selectable.
pub fn calendar(subject: Subject, text: &str, year: i32, month: u32, today: NaiveDate) -> Panel {
    let first =
        NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(today.with_day(1).unwrap_or(today));
    let this_month = today.with_day(1).unwrap_or(today);
    let last_month = this_month
        .checked_add_months(Months::new(MAX_MONTHS_AHEAD))
        .unwrap_or(this_month);
    let first = first.clamp(this_month, last_month);

    let nav = |enabled: bool, label: &str, target: Option<NaiveDate>| match (enabled, target) {
        (true, Some(d)) => button(label, Action::Calendar(subject, d.year(), d.month())),
        _ => blank(),
    };
    let mut rows = vec![
        vec![
            nav(
                first > this_month,
                "‹",
                first.checked_sub_months(Months::new(1)),
            ),
            button(first.format("%B %Y").to_string(), Action::Noop),
            nav(
                first < last_month,
                "›",
                first.checked_add_months(Months::new(1)),
            ),
        ],
        ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
            .into_iter()
            .map(|d| button(d, Action::Noop))
            .collect(),
    ];

    let mut week: Vec<InlineKeyboardButton> = (0..first.weekday().num_days_from_monday())
        .map(|_| blank())
        .collect();
    let mut day = first;
    while day.month() == first.month() {
        week.push(if day < today {
            button("·", Action::Noop)
        } else if day == today {
            button(format!("{}•", day.day()), Action::Day(subject, day))
        } else {
            button(day.day().to_string(), Action::Day(subject, day))
        });
        if week.len() == 7 {
            rows.push(std::mem::take(&mut week));
        }
        day += Duration::days(1);
    }
    if !week.is_empty() {
        week.resize_with(7, blank);
        rows.push(week);
    }
    rows.push(back_or_cancel(subject, Action::When(subject)));

    Panel {
        text: format!("📝 {text}\n\nPick a date:"),
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

/// Step 3: hour of the day.
pub fn hours(subject: Subject, text: &str, date: NaiveDate, now: NaiveDateTime) -> Panel {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = (0..24)
        .collect::<Vec<u32>>()
        .chunks(6)
        .map(|chunk| {
            chunk
                .iter()
                .map(|&h| {
                    let past = date == now.date() && h < now.hour();
                    if past {
                        button("·", Action::Noop)
                    } else {
                        button(format!("{h:02}"), Action::Hour(subject, date, h))
                    }
                })
                .collect()
        })
        .collect();
    rows.push(back_or_cancel(
        subject,
        Action::Calendar(subject, date.year(), date.month()),
    ));

    Panel {
        text: format!(
            "📝 {text}\n🗓 {}\n\nPick the hour:",
            date.format("%a %-d %b %Y")
        ),
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

/// Step 4: minute, in 5-minute steps.
pub fn minutes(
    subject: Subject,
    text: &str,
    date: NaiveDate,
    hour: u32,
    now: NaiveDateTime,
) -> Panel {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = (0..60)
        .step_by(5)
        .collect::<Vec<u32>>()
        .chunks(4)
        .map(|chunk| {
            chunk
                .iter()
                .map(|&m| {
                    let at = date.and_time(time(hour, m));
                    if at <= now {
                        button("·", Action::Noop)
                    } else {
                        button(format!("{hour:02}:{m:02}"), Action::At(subject, at))
                    }
                })
                .collect()
        })
        .collect();
    rows.push(back_or_cancel(subject, Action::Day(subject, date)));

    Panel {
        text: format!(
            "📝 {text}\n🗓 {}\n\nPick the time:",
            date.format("%a %-d %b %Y")
        ),
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

/// Final step: how often. Tapping an option saves the reminder.
pub fn repeat(subject: Subject, text: &str, at: NaiveDateTime) -> Panel {
    let option = |label: String, r: Recurrence| button(label, Action::Save(subject, at, r));
    let mut rows = vec![
        vec![
            option("Once".into(), Recurrence::Once),
            option("Every day".into(), Recurrence::Daily),
            option("Weekdays".into(), Recurrence::Weekdays),
        ],
        vec![
            option(format!("Every {}", at.format("%a")), Recurrence::Weekly),
            option(format!("Monthly · {}", at.day()), Recurrence::Monthly),
            option(
                format!("Yearly · {}", at.format("%-d %b")),
                Recurrence::Yearly,
            ),
        ],
    ];
    let back = match subject {
        Subject::Draft(_) => Action::When(subject),
        Subject::Reminder(id) => Action::Manage(id),
    };
    rows.push(back_or_cancel(subject, back));

    Panel {
        text: format!("📝 {text}\n🗓 {}\n\nHow often?", format_when(at)),
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

fn summary(reminder: &Reminder, user: &User, now: DateTime<Utc>) -> String {
    let local = user.tz.local(reminder.due_at);
    format!(
        "📝 {}\n🗓 {} ({})\n🔁 {}",
        reminder.description,
        format_when(local),
        relative(now, reminder.due_at),
        reminder.recurrence.describe(local)
    )
}

pub fn saved(reminder: &Reminder, user: &User, now: DateTime<Utc>, heading: &str) -> Panel {
    Panel {
        text: format!("{heading}\n\n{}", summary(reminder, user, now)),
        keyboard: InlineKeyboardMarkup::new([[
            button("✏️ Edit", Action::Manage(reminder.id)),
            button("↩️ Undo", Action::Delete(reminder.id)),
        ]]),
    }
}

pub fn manage(reminder: &Reminder, user: &User, now: DateTime<Utc>) -> Panel {
    let local = user.tz.local(reminder.due_at);
    let subject = Subject::Reminder(reminder.id);
    Panel {
        text: summary(reminder, user, now),
        keyboard: InlineKeyboardMarkup::new([
            vec![
                button("🕐 Change time", Action::When(subject)),
                button("🔁 Change repeat", Action::At(subject, local)),
            ],
            vec![
                button("🗑 Delete", Action::Delete(reminder.id)),
                button("📋 All reminders", Action::List),
            ],
        ]),
    }
}

pub fn list(reminders: &[Reminder], user: &User) -> Panel {
    if reminders.is_empty() {
        return Panel {
            text: "You have no reminders. Send me any message to create one.".into(),
            keyboard: InlineKeyboardMarkup::default(),
        };
    }

    let mut text = format!("Your reminders ({}):\n", user.tz);
    let mut rows = Vec::new();
    for (index, reminder) in reminders.iter().take(MAX_LISTED).enumerate() {
        let local = user.tz.local(reminder.due_at);
        text.push_str(&format!(
            "\n{}. {}\n    {} · {}",
            index + 1,
            reminder.description,
            format_when(local),
            reminder.recurrence.describe(local)
        ));
        rows.push(vec![button(
            format!("{}. {}", index + 1, truncate(&reminder.description, 40)),
            Action::Manage(reminder.id),
        )]);
    }
    if reminders.len() > MAX_LISTED {
        text.push_str(&format!("\n\n…and {} more.", reminders.len() - MAX_LISTED));
    }
    text.push_str("\n\nTap a reminder to change or delete it.");
    Panel {
        text,
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

pub fn nag_keyboard(nagging_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        vec![button("✅ Done", Action::Done(nagging_id))],
        vec![
            button("💤 10 min", Action::Snooze(nagging_id, Snooze::TenMinutes)),
            button("💤 1 hour", Action::Snooze(nagging_id, Snooze::OneHour)),
            button(
                "💤 Tomorrow",
                Action::Snooze(nagging_id, Snooze::TomorrowMorning),
            ),
        ],
    ])
}

fn time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time")
}

fn truncate_to_minute(at: NaiveDateTime) -> NaiveDateTime {
    at.with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(at)
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else {
        format!("{}…", text.chars().take(max_chars - 1).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    fn actions(panel: &Panel) -> Vec<Action> {
        panel
            .keyboard
            .inline_keyboard
            .iter()
            .flatten()
            .filter_map(|b| match &b.kind {
                teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => {
                    Action::parse(data)
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn calendar_disables_past_days_and_previous_month() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 13).unwrap();
        let panel = calendar(Subject::Draft(1), "x", 2026, 10, today);
        let days: Vec<NaiveDate> = actions(&panel)
            .into_iter()
            .filter_map(|a| match a {
                Action::Day(_, d) => Some(d),
                _ => None,
            })
            .collect();
        assert_eq!(days.first(), Some(&today));
        assert_eq!(days.last(), NaiveDate::from_ymd_opt(2026, 10, 31).as_ref());
        assert!(!actions(&panel).contains(&Action::Calendar(Subject::Draft(1), 2026, 9)));
        assert!(actions(&panel).contains(&Action::Calendar(Subject::Draft(1), 2026, 11)));
        assert!(
            panel
                .keyboard
                .inline_keyboard
                .iter()
                .all(|row| row.len() <= 7)
        );
    }

    #[test]
    fn minute_picker_hides_past_times() {
        let now = local(2026, 10, 13, 10, 32);
        let panel = minutes(Subject::Draft(1), "x", now.date(), 10, now);
        let first = actions(&panel).into_iter().find_map(|a| match a {
            Action::At(_, at) => Some(at),
            _ => None,
        });
        assert_eq!(first, Some(local(2026, 10, 13, 10, 35)));
    }

    #[test]
    fn quick_choices_move_evening_to_tomorrow_when_late() {
        let panel = when(Subject::Draft(1), "x", local(2026, 10, 13, 21, 0));
        assert!(
            actions(&panel).contains(&Action::At(Subject::Draft(1), local(2026, 10, 14, 20, 0)))
        );
    }

    #[test]
    fn relative_times() {
        let t = local(2026, 1, 1, 0, 0).and_utc();
        assert_eq!(relative(t, t + Duration::minutes(5)), "in 5 min");
        assert_eq!(relative(t, t + Duration::minutes(200)), "in 3 h 20 min");
        assert_eq!(relative(t, t + Duration::hours(2)), "in 2 h");
        assert_eq!(relative(t, t + Duration::days(4)), "in 4 days");
    }
}
