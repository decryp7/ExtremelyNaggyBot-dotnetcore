//! The reminder flow: what to do with text, locations and button presses.
//!
//! Handlers return replies instead of calling Telegram, so the flow can be tested against an
//! in-memory database.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use teloxide::types::InlineKeyboardMarkup;

use crate::callback::{Action, Snooze, Subject};
use crate::db::Db;
use crate::model::{User, UserTz};
use crate::panels::{self, Panel};
use crate::time_input;

/// A typed time is applied to the user's draft only if the draft is this recent.
const TYPED_TIME_WINDOW: Duration = Duration::minutes(30);
/// How long a draft waits for an unregistered user to choose their timezone.
const TIMEZONE_WINDOW: Duration = Duration::hours(24);
/// Short unrecognised text from an unregistered user is treated as a failed timezone answer.
const MAX_TIMEZONE_ANSWER_CHARS: usize = 30;
const MAX_DESCRIPTION_CHARS: usize = 1000;
const SNOOZE_MORNING: NaiveTime = NaiveTime::from_hms_opt(9, 0, 0).expect("valid time");

/// The chat a message came from. `chat_id` identifies the user in the database (in private
/// chats it equals the Telegram user id).
pub struct Sender {
    pub chat_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub private: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Markup {
    None,
    Inline(InlineKeyboardMarkup),
    /// One-tap "share my location" keyboard (private chats only).
    AskLocation,
    RemoveKeyboard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub text: String,
    pub markup: Markup,
}

impl Reply {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            markup: Markup::None,
        }
    }
}

impl From<Panel> for Reply {
    fn from(panel: Panel) -> Self {
        Self {
            text: panel.text,
            markup: Markup::Inline(panel.keyboard),
        }
    }
}

/// Response to a button press: an optional popup and an optional edit of the message.
#[derive(Default)]
pub struct CallbackReply {
    pub toast: Option<String>,
    pub edit: Option<Panel>,
}

impl CallbackReply {
    fn edit(panel: Panel) -> Self {
        Self {
            toast: None,
            edit: Some(panel),
        }
    }

    fn toast(text: impl Into<String>) -> Self {
        Self {
            toast: Some(text.into()),
            edit: None,
        }
    }

    /// Replaces the message text and removes its buttons.
    fn finish(text: impl Into<String>, toast: Option<&str>) -> Self {
        Self {
            toast: toast.map(str::to_owned),
            edit: Some(Panel {
                text: text.into(),
                keyboard: InlineKeyboardMarkup::default(),
            }),
        }
    }
}

/// Plain text or a forwarded message.
pub async fn on_text(
    db: &Db,
    sender: &Sender,
    text: &str,
    forwarded: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Reply>> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(Vec::new());
    }

    let Some(user) = db.user(sender.chat_id).await? else {
        // Not registered: this is either the answer to "which timezone?" or a new reminder.
        if !forwarded
            && db
                .pending_draft(sender.chat_id, now - TIMEZONE_WINDOW)
                .await?
                .is_some()
        {
            match UserTz::parse(text) {
                Ok(tz) => return set_timezone(db, sender, tz, now).await,
                Err(error) if text.chars().count() <= MAX_TIMEZONE_ANSWER_CHARS => {
                    return Ok(vec![ask_timezone(sender, &error.to_string())]);
                }
                Err(_) => {}
            }
        }
        let draft = db.create_draft(sender.chat_id, clamp(text), now).await?;
        return Ok(vec![ask_timezone(
            sender,
            &format!("📝 {}\n\nFirst, which timezone are you in?", draft.text),
        )]);
    };

    if !forwarded
        && let Some(draft) = db.pending_draft(user.id, now - TYPED_TIME_WINDOW).await?
        && let Some(at) = time_input::parse(text, user.tz, now)
    {
        return Ok(vec![
            panels::repeat(Subject::Draft(draft.id), &draft.text, at).into(),
        ]);
    }
    start_reminder(db, &user, text, now).await
}

/// Starts the "when?" flow for `text`.
pub async fn start_reminder(
    db: &Db,
    user: &User,
    text: &str,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Reply>> {
    let draft = db.create_draft(user.id, clamp(text), now).await?;
    Ok(vec![
        panels::when(Subject::Draft(draft.id), &draft.text, user.tz.local(now)).into(),
    ])
}

/// Starts a reminder for a sender who may not be registered yet (used by `/remindme`).
pub async fn start_reminder_for(
    db: &Db,
    sender: &Sender,
    text: &str,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Reply>> {
    match db.user(sender.chat_id).await? {
        Some(user) => start_reminder(db, &user, text, now).await,
        None => on_text(db, sender, text, true, now).await,
    }
}

pub async fn on_location(
    db: &Db,
    sender: &Sender,
    latitude: f64,
    longitude: f64,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Reply>> {
    let tz =
        tokio::task::spawn_blocking(move || UserTz::from_location(latitude, longitude)).await?;
    match tz {
        Some(tz) => set_timezone(db, sender, tz, now).await,
        None => Ok(vec![ask_timezone(
            sender,
            "I couldn't work out the timezone for that location.",
        )]),
    }
}

/// Registers the sender (or updates their timezone) and resumes any pending reminder.
pub async fn set_timezone(
    db: &Db,
    sender: &Sender,
    tz: UserTz,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Reply>> {
    let user = User {
        id: sender.chat_id,
        first_name: sender.first_name.clone(),
        last_name: sender.last_name.clone(),
        tz,
    };
    db.upsert_user(user).await?;

    let mut replies = vec![Reply {
        text: format!("🌍 Timezone set to {tz}."),
        markup: Markup::RemoveKeyboard,
    }];
    match db
        .pending_draft(sender.chat_id, now - TIMEZONE_WINDOW)
        .await?
    {
        Some(draft) => {
            replies.push(panels::when(Subject::Draft(draft.id), &draft.text, tz.local(now)).into())
        }
        None => replies.push(Reply::text("Send me any message to create a reminder.")),
    }
    Ok(replies)
}

pub fn ask_timezone(sender: &Sender, intro: &str) -> Reply {
    if sender.private {
        Reply {
            text: format!(
                "{intro}\n\nTap \"📍 Share my location\" below, or type your city (London), timezone (Asia/Singapore) or UTC offset (+8)."
            ),
            markup: Markup::AskLocation,
        }
    } else {
        Reply::text(format!(
            "{intro}\n\nSet it with /timezone followed by your city, timezone or UTC offset, e.g. /timezone London."
        ))
    }
}

pub async fn on_callback(
    db: &Db,
    user_id: i64,
    data: &str,
    message_text: Option<&str>,
    now: DateTime<Utc>,
) -> anyhow::Result<CallbackReply> {
    let Some(action) = Action::parse(data) else {
        return Ok(CallbackReply::toast("This button is no longer valid."));
    };

    // Buttons on reminder messages.
    let nag_text = message_text.map_or("", |t| t.trim_start_matches("⏰").trim_start());
    match action {
        Action::Noop => return Ok(CallbackReply::default()),
        Action::Done(id) | Action::LegacyNaggingAck(id) => {
            return Ok(if db.acknowledge_nagging(user_id, id).await? {
                CallbackReply::finish(format!("✅ {nag_text}"), Some("Done ✅"))
            } else {
                CallbackReply::finish(message_text.unwrap_or_default(), Some("Already done"))
            });
        }
        Action::LegacyReminderAck(id) => {
            return Ok(if db.acknowledge_reminder(user_id, id).await? {
                CallbackReply::finish(format!("✅ {nag_text}"), Some("Done ✅"))
            } else {
                CallbackReply::finish(message_text.unwrap_or_default(), Some("Already done"))
            });
        }
        _ => {}
    }

    let Some(user) = db.user(user_id).await? else {
        return Ok(CallbackReply::toast(
            "Please set your timezone first: send /start.",
        ));
    };
    let now_local = user.tz.local(now);

    Ok(match action {
        Action::Snooze(id, snooze) => {
            let until = match snooze {
                Snooze::TenMinutes => now + Duration::minutes(10),
                Snooze::OneHour => now + Duration::hours(1),
                Snooze::TomorrowMorning => user
                    .tz
                    .utc((now_local.date() + Duration::days(1)).and_time(SNOOZE_MORNING)),
            };
            if db.snooze_nagging(user_id, id, until).await? {
                CallbackReply::finish(
                    format!(
                        "💤 {nag_text}\nSnoozed until {}",
                        panels::format_when(user.tz.local(until))
                    ),
                    Some("Snoozed 💤"),
                )
            } else {
                CallbackReply::finish(message_text.unwrap_or_default(), Some("Already done"))
            }
        }

        Action::When(subject) => match subject_text(db, user_id, subject).await? {
            Some(text) => CallbackReply::edit(panels::when(subject, &text, now_local)),
            None => gone(message_text),
        },
        Action::Calendar(subject, year, month) => match subject_text(db, user_id, subject).await? {
            Some(text) => CallbackReply::edit(panels::calendar(
                subject,
                &text,
                year,
                month,
                now_local.date(),
            )),
            None => gone(message_text),
        },
        Action::Day(subject, date) => match subject_text(db, user_id, subject).await? {
            Some(text) => CallbackReply::edit(panels::hours(subject, &text, date, now_local)),
            None => gone(message_text),
        },
        Action::Hour(subject, date, hour) => match subject_text(db, user_id, subject).await? {
            Some(text) => {
                CallbackReply::edit(panels::minutes(subject, &text, date, hour, now_local))
            }
            None => gone(message_text),
        },
        Action::At(subject, at) => match subject_text(db, user_id, subject).await? {
            Some(text) => CallbackReply::edit(panels::repeat(subject, &text, at)),
            None => gone(message_text),
        },

        Action::Save(subject, at, recurrence) => {
            let Some(due) = recurrence.first_due(at, user.tz, now) else {
                let Some(text) = subject_text(db, user_id, subject).await? else {
                    return Ok(gone(message_text));
                };
                return Ok(CallbackReply {
                    toast: Some("That time has already passed. Pick another.".into()),
                    edit: Some(panels::when(subject, &text, now_local)),
                });
            };
            let (saved, heading) = match subject {
                Subject::Draft(id) => (
                    db.save_draft(user_id, id, due, recurrence).await?,
                    "✅ Reminder saved",
                ),
                Subject::Reminder(id) => (
                    db.update_reminder(user_id, id, None, due, recurrence)
                        .await?,
                    "✅ Reminder updated",
                ),
            };
            match saved {
                Some(reminder) => CallbackReply {
                    toast: Some("Saved ✅".into()),
                    edit: Some(panels::saved(&reminder, &user, now, heading)),
                },
                None => gone(message_text),
            }
        }

        Action::CancelDraft(id) => {
            db.delete_draft(user_id, id).await?;
            CallbackReply::finish("✖ Cancelled.", None)
        }
        Action::Manage(id) => match db.reminder(user_id, id).await? {
            Some(reminder) => CallbackReply::edit(panels::manage(&reminder, &user, now)),
            None => gone(message_text),
        },
        Action::Delete(id) => match db.reminder(user_id, id).await? {
            Some(reminder) => {
                db.delete_reminder(user_id, id).await?;
                CallbackReply::finish(
                    format!("🗑 Removed: {}", reminder.description),
                    Some("Removed"),
                )
            }
            None => gone(message_text),
        },
        Action::List => {
            let reminders = db.reminders_for_user(user_id).await?;
            CallbackReply::edit(panels::list(&reminders, &user))
        }

        Action::Noop
        | Action::Done(_)
        | Action::LegacyNaggingAck(_)
        | Action::LegacyReminderAck(_) => {
            unreachable!("handled above")
        }
    })
}

async fn subject_text(db: &Db, user_id: i64, subject: Subject) -> anyhow::Result<Option<String>> {
    Ok(match subject {
        Subject::Draft(id) => db.draft(user_id, id).await?.map(|d| d.text),
        Subject::Reminder(id) => db.reminder(user_id, id).await?.map(|r| r.description),
    })
}

/// The draft or reminder behind a button no longer exists (saved, cancelled or deleted).
fn gone(message_text: Option<&str>) -> CallbackReply {
    CallbackReply::finish(
        message_text.unwrap_or("This reminder is no longer available."),
        Some("This reminder is no longer available."),
    )
}

fn clamp(text: &str) -> String {
    text.chars().take(MAX_DESCRIPTION_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Recurrence;
    use chrono::{NaiveDate, NaiveDateTime};
    use chrono_tz::Tz;
    use teloxide::types::InlineKeyboardButtonKind;

    const SGT: UserTz = UserTz::Named(Tz::Asia__Singapore);

    fn local(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    fn now() -> DateTime<Utc> {
        SGT.utc(local(2026, 10, 13, 10, 30))
    }

    fn sender() -> Sender {
        Sender {
            chat_id: 7,
            first_name: "Ada".into(),
            last_name: String::new(),
            private: true,
        }
    }

    fn buttons(markup: &Markup) -> Vec<(String, Action)> {
        let Markup::Inline(keyboard) = markup else {
            return Vec::new();
        };
        keyboard
            .inline_keyboard
            .iter()
            .flatten()
            .filter_map(|b| match &b.kind {
                InlineKeyboardButtonKind::CallbackData(data) => {
                    Some((b.text.clone(), Action::parse(data)?))
                }
                _ => None,
            })
            .collect()
    }

    fn find(markup: &Markup, label: &str) -> Action {
        buttons(markup)
            .into_iter()
            .find(|(text, _)| text == label)
            .unwrap_or_else(|| panic!("no button {label:?} in {:?}", buttons(markup)))
            .1
    }

    async fn press(db: &Db, action: Action) -> CallbackReply {
        on_callback(db, 7, &action.encode(), Some("message"), now())
            .await
            .unwrap()
    }

    fn edit_markup(reply: &CallbackReply) -> Markup {
        Markup::Inline(reply.edit.as_ref().unwrap().keyboard.clone())
    }

    #[tokio::test]
    async fn new_user_is_asked_for_timezone_then_continues() {
        let db = Db::open_in_memory().unwrap();
        let replies = on_text(&db, &sender(), "call mum", false, now())
            .await
            .unwrap();
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].markup, Markup::AskLocation);

        let replies = on_text(&db, &sender(), "Atlantis", false, now())
            .await
            .unwrap();
        assert!(
            replies[0].text.contains("don't recognise"),
            "{}",
            replies[0].text
        );

        let replies = on_text(&db, &sender(), "Singapore", false, now())
            .await
            .unwrap();
        assert!(replies[0].text.contains("Asia/Singapore"));
        assert_eq!(replies[0].markup, Markup::RemoveKeyboard);
        assert!(replies[1].text.contains("call mum"));
        assert!(replies[1].text.contains("When should I remind you?"));
    }

    #[tokio::test]
    async fn quick_choice_flow_saves_in_two_taps() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();

        let replies = on_text(&db, &sender(), "call mum", false, now())
            .await
            .unwrap();
        let tomorrow = find(&replies[0].markup, "Tomorrow 09:00");

        let repeat = press(&db, tomorrow).await;
        let daily = find(&edit_markup(&repeat), "Every day");

        let saved = press(&db, daily).await;
        assert!(
            saved
                .edit
                .as_ref()
                .unwrap()
                .text
                .starts_with("✅ Reminder saved")
        );

        let reminders = db.reminders_for_user(7).await.unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].description, "call mum");
        assert_eq!(reminders[0].recurrence, Recurrence::Daily);
        assert_eq!(SGT.local(reminders[0].due_at), local(2026, 10, 14, 9, 0));

        // Pressing the same button again does not create a duplicate.
        press(&db, daily).await;
        assert_eq!(db.reminders_for_user(7).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn calendar_flow_picks_date_hour_and_minute() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();
        let replies = on_text(&db, &sender(), "dentist", false, now())
            .await
            .unwrap();

        let calendar = press(&db, find(&replies[0].markup, "📅 Pick a date")).await;
        let next_month = press(&db, find(&edit_markup(&calendar), "›")).await;
        let hours = press(&db, find(&edit_markup(&next_month), "2")).await;
        let minutes = press(&db, find(&edit_markup(&hours), "14")).await;
        let repeat = press(&db, find(&edit_markup(&minutes), "14:45")).await;
        press(&db, find(&edit_markup(&repeat), "Once")).await;

        let reminders = db.reminders_for_user(7).await.unwrap();
        assert_eq!(SGT.local(reminders[0].due_at), local(2026, 11, 2, 14, 45));
        assert_eq!(reminders[0].recurrence, Recurrence::Once);
    }

    #[tokio::test]
    async fn typed_time_answers_the_when_question() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();
        on_text(&db, &sender(), "pay rent", false, now())
            .await
            .unwrap();

        let replies = on_text(&db, &sender(), "fri 6pm", false, now())
            .await
            .unwrap();
        assert!(
            replies[0].text.contains("Fri 16 Oct 2026, 18:00"),
            "{}",
            replies[0].text
        );
        press(&db, find(&replies[0].markup, "Monthly · 16")).await;

        let reminders = db.reminders_for_user(7).await.unwrap();
        assert_eq!(reminders[0].description, "pay rent");
        assert_eq!(reminders[0].recurrence, Recurrence::Monthly);
    }

    #[tokio::test]
    async fn edit_and_delete_from_the_list() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();
        let replies = on_text(&db, &sender(), "stretch", false, now())
            .await
            .unwrap();
        let repeat = press(&db, find(&replies[0].markup, "1 hour")).await;
        press(&db, find(&edit_markup(&repeat), "Weekdays")).await;
        let reminder = db.reminders_for_user(7).await.unwrap().remove(0);

        let list = press(&db, Action::List).await;
        let manage = press(&db, find(&edit_markup(&list), "1. stretch")).await;
        let repeat = press(&db, find(&edit_markup(&manage), "🔁 Change repeat")).await;
        press(&db, find(&edit_markup(&repeat), "Every Tue")).await;
        assert_eq!(
            db.reminder(7, reminder.id)
                .await
                .unwrap()
                .unwrap()
                .recurrence,
            Recurrence::Weekly
        );

        let deleted = press(&db, Action::Delete(reminder.id)).await;
        assert!(deleted.edit.unwrap().text.contains("Removed: stretch"));
        assert!(db.reminders_for_user(7).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn cancel_discards_the_draft() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();
        let replies = on_text(&db, &sender(), "nothing", false, now())
            .await
            .unwrap();
        let quick = find(&replies[0].markup, "15 min");
        press(&db, find(&replies[0].markup, "✖ Cancel")).await;
        let reply = press(&db, quick).await;
        assert_eq!(
            reply.toast.as_deref(),
            Some("This reminder is no longer available.")
        );
    }

    #[tokio::test]
    async fn snooze_and_done_on_a_nag() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();
        let reminder = db
            .add_reminder(7, "pills".into(), now(), Recurrence::Once)
            .await
            .unwrap();
        let nagging = db
            .fire_reminder(reminder.id, now(), now())
            .await
            .unwrap()
            .unwrap();

        let reply = on_callback(
            &db,
            7,
            &Action::Snooze(nagging.id, Snooze::TomorrowMorning).encode(),
            Some("⏰ pills"),
            now(),
        )
        .await
        .unwrap();
        assert!(
            reply
                .edit
                .unwrap()
                .text
                .contains("Snoozed until Wed 14 Oct 2026, 09:00")
        );
        assert!(db.due_naggings(now()).await.unwrap().is_empty());

        let reply = on_callback(
            &db,
            7,
            &Action::Done(nagging.id).encode(),
            Some("⏰ pills"),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(reply.edit.unwrap().text, "✅ pills");
        assert!(
            db.due_naggings(now() + Duration::days(2))
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn other_users_cannot_press_your_buttons() {
        let db = Db::open_in_memory().unwrap();
        set_timezone(&db, &sender(), SGT, now()).await.unwrap();
        let mut other = sender();
        other.chat_id = 8;
        set_timezone(&db, &other, SGT, now()).await.unwrap();

        let replies = on_text(&db, &sender(), "secret", false, now())
            .await
            .unwrap();
        let quick = find(&replies[0].markup, "15 min");
        let reply = on_callback(&db, 8, &quick.encode(), None, now())
            .await
            .unwrap();
        assert_eq!(
            reply.toast.as_deref(),
            Some("This reminder is no longer available.")
        );
    }
}
