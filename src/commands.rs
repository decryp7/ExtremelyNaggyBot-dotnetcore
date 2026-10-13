//! Bot commands: parsing and execution. Execution returns the reply text and does not
//! talk to Telegram, so it can be tested against an in-memory database.

use std::sync::LazyLock;

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use teloxide::utils::command::BotCommands;

use crate::db::{Db, Registration};
use crate::model::{InputError, Recurrence, User, UtcOffset};

const DISPLAY_TIME_FORMAT: &str = "%d/%m/%Y %H:%M";
const SOURCE_URL: &str = "https://dev.decryptology.net/decryp7/ExtremelyNaggyBot";

#[derive(BotCommands, Clone, Debug, PartialEq, Eq)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(hide)]
    Start,
    #[command(description = "show this help.")]
    Help,
    #[command(description = "information about this bot.")]
    About,
    #[command(
        description = "<utc_offset> — register or change your timezone, e.g. /register +8 or /register +5:30."
    )]
    Register(String),
    #[command(description = "remove yourself and all your reminders.")]
    Unregister,
    #[command(
        description = "<description> <dd/mm/yyyy> <HH:MM> [once|daily|weekly|monthly] — add a reminder (24-hour time, once by default)."
    )]
    RemindMe(String),
    #[command(description = "list your reminders.")]
    ListReminders,
    #[command(description = "<id> — remove a reminder.")]
    RemoveReminder(String),
    #[command(description = "remove all your reminders.")]
    RemoveAllReminders,
}

/// Who sent the command. `chat_id` identifies the user in the database, as it did in the
/// original bot (in private chats it equals the Telegram user id).
pub struct Sender {
    pub chat_id: i64,
    pub first_name: String,
    pub last_name: String,
}

pub struct BotInfo {
    pub name: String,
    pub username: String,
}

pub async fn execute(
    command: Command,
    sender: &Sender,
    bot: &BotInfo,
    db: &Db,
) -> anyhow::Result<String> {
    let name = &sender.first_name;
    Ok(match command {
        Command::Start => format!(
            "Hello {name}, {} at your service.\n\nStart with /register <utc_offset>, then add reminders with /remindme. Send /help for all commands.",
            bot.name
        ),
        Command::Help => Command::descriptions().username(&bot.username).to_string(),
        Command::About => format!(
            "Hello {name}, I am an extremely naggy reminder bot. I keep reminding you every minute until you acknowledge.\n\nBug reports: @decryp7\nSource code: {SOURCE_URL}"
        ),
        Command::Register(args) => register(&args, sender, db).await?,
        Command::Unregister => {
            if db.delete_user(sender.chat_id).await? {
                format!("Goodbye {name}, I have removed you and all your reminders.")
            } else {
                format!("{name}, you are not registered.")
            }
        }
        Command::RemindMe(args) => match registered_user(sender, db).await? {
            Ok(user) => remind_me(&args, &user, Utc::now(), db).await?,
            Err(reply) => reply,
        },
        Command::ListReminders => match registered_user(sender, db).await? {
            Ok(user) => list_reminders(&user, db).await?,
            Err(reply) => reply,
        },
        Command::RemoveReminder(args) => {
            let Ok(id) = args.trim().trim_start_matches('#').parse::<i64>() else {
                return Ok(
                    "Usage: /removereminder <id>. Use /listreminders to see the ids.".into(),
                );
            };
            if db.delete_reminder(sender.chat_id, id).await? {
                format!("{name}, I have removed reminder #{id}.")
            } else {
                format!("{name}, you do not have a reminder #{id}.")
            }
        }
        Command::RemoveAllReminders => match db.delete_all_reminders(sender.chat_id).await? {
            0 => format!("{name}, you do not have any reminders."),
            1 => format!("{name}, I have removed your reminder."),
            n => format!("{name}, I have removed all {n} of your reminders."),
        },
    })
}

async fn registered_user(sender: &Sender, db: &Db) -> anyhow::Result<Result<User, String>> {
    Ok(db.user(sender.chat_id).await?.ok_or_else(|| {
        format!(
            "{}, please register first with /register <utc_offset>, e.g. /register +8.",
            sender.first_name
        )
    }))
}

async fn register(args: &str, sender: &Sender, db: &Db) -> anyhow::Result<String> {
    if args.trim().is_empty() {
        return Ok("Usage: /register <utc_offset>, e.g. /register +8 or /register -3:30.".into());
    }
    let utc_offset = match args.parse::<UtcOffset>() {
        Ok(offset) => offset,
        Err(e) => return Ok(e.to_string()),
    };
    let user = User {
        id: sender.chat_id,
        first_name: sender.first_name.clone(),
        last_name: sender.last_name.clone(),
        utc_offset,
    };
    Ok(match db.upsert_user(user).await? {
        Registration::Created => format!(
            "Hello {}, I have registered you with timezone {utc_offset}.",
            sender.first_name
        ),
        Registration::Updated => format!(
            "{}, I have updated your timezone to {utc_offset}.",
            sender.first_name
        ),
    })
}

async fn remind_me(args: &str, user: &User, now: DateTime<Utc>, db: &Db) -> anyhow::Result<String> {
    let request = match ReminderRequest::parse(args, user.utc_offset, now) {
        Ok(Some(request)) => request,
        Ok(None) => return Ok(format!("Usage: /remindme {}", remind_me_usage())),
        Err(e) => return Ok(e.to_string()),
    };
    let reminder = db
        .add_reminder(
            user.id,
            request.description,
            request.due_at,
            request.recurrence,
        )
        .await?;
    Ok(format!(
        "{}, I will remind you about \"{}\" on {} ({}), {}. Reminder id: #{}.",
        user.first_name,
        reminder.description,
        user.local_time(reminder.due_at).format(DISPLAY_TIME_FORMAT),
        user.utc_offset,
        reminder.recurrence,
        reminder.id
    ))
}

async fn list_reminders(user: &User, db: &Db) -> anyhow::Result<String> {
    let reminders = db.reminders_for_user(user.id).await?;
    if reminders.is_empty() {
        return Ok(format!(
            "{}, you do not have any reminders.",
            user.first_name
        ));
    }
    let lines: Vec<String> = reminders
        .iter()
        .map(|r| {
            format!(
                "#{} {} — {} ({})",
                r.id,
                r.description,
                user.local_time(r.due_at).format(DISPLAY_TIME_FORMAT),
                r.recurrence
            )
        })
        .collect();
    Ok(format!(
        "{}, your reminders ({}):\n{}",
        user.first_name,
        user.utc_offset,
        lines.join("\n")
    ))
}

fn remind_me_usage() -> &'static str {
    "<description> <dd/mm/yyyy> <HH:MM> [once|daily|weekly|monthly]\nExample: /remindme Take pills 25/12/2026 08:00 daily"
}

#[derive(Debug, PartialEq, Eq)]
struct ReminderRequest {
    description: String,
    due_at: DateTime<Utc>,
    recurrence: Recurrence,
}

impl ReminderRequest {
    /// Returns `Ok(None)` when the arguments do not have the expected shape at all.
    /// A recurring reminder whose first occurrence has passed is moved to the next one.
    fn parse(
        args: &str,
        offset: UtcOffset,
        now: DateTime<Utc>,
    ) -> Result<Option<Self>, InputError> {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?is)^\s*(?P<description>.+?)\s+(?P<date>\d{1,2}/\d{1,2}/\d{4})\s+(?P<time>\d{1,2}:\d{2})(?:\s+(?P<recurrence>\S+))?\s*$",
            )
            .expect("valid regex")
        });

        let Some(caps) = PATTERN.captures(args) else {
            return Ok(None);
        };

        let description = caps["description"].trim().to_owned();
        if description.is_empty() {
            return Err(InputError::EmptyDescription);
        }
        let recurrence = caps
            .name("recurrence")
            .map_or(Ok(Recurrence::Once), |m| m.as_str().parse())?;

        let local = NaiveDateTime::parse_from_str(
            &format!("{} {}", &caps["date"], &caps["time"]),
            "%d/%m/%Y %H:%M",
        )
        .map_err(|_| InputError::InvalidDateTime)?;
        let mut due_at = local
            .and_local_timezone(offset.fixed())
            .single()
            .ok_or(InputError::InvalidDateTime)?
            .with_timezone(&Utc);

        if due_at <= now {
            due_at = recurrence
                .next_after(due_at, now)
                .ok_or(InputError::InPast)?;
        }

        Ok(Some(Self {
            description,
            due_at,
            recurrence,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    fn sgt() -> UtcOffset {
        UtcOffset::from_minutes(480).unwrap()
    }

    #[test]
    fn parses_commands() {
        assert_eq!(
            Command::parse("/start", "naggybot").unwrap(),
            Command::Start
        );
        assert_eq!(
            Command::parse("/listreminders@naggybot", "naggybot").unwrap(),
            Command::ListReminders
        );
        assert_eq!(
            Command::parse("/remindme pay rent 01/11/2026 09:00 monthly", "naggybot").unwrap(),
            Command::RemindMe("pay rent 01/11/2026 09:00 monthly".into())
        );
        assert_eq!(
            Command::parse("/register", "naggybot").unwrap(),
            Command::Register(String::new())
        );
        assert!(Command::parse("/unknown", "naggybot").is_err());
    }

    #[test]
    fn parses_reminder_in_users_timezone() {
        let now = utc(2026, 10, 1, 0, 0);
        let request = ReminderRequest::parse("Take my pills 25/12/2026 08:00 Daily", sgt(), now)
            .unwrap()
            .unwrap();
        assert_eq!(
            request,
            ReminderRequest {
                description: "Take my pills".into(),
                due_at: utc(2026, 12, 25, 0, 0),
                recurrence: Recurrence::Daily,
            }
        );
    }

    #[test]
    fn recurrence_defaults_to_once() {
        let now = utc(2026, 10, 1, 0, 0);
        let request = ReminderRequest::parse("dentist 2/11/2026 9:30", sgt(), now)
            .unwrap()
            .unwrap();
        assert_eq!(request.recurrence, Recurrence::Once);
        assert_eq!(request.due_at, utc(2026, 11, 2, 1, 30));
    }

    #[test]
    fn rejects_bad_reminders() {
        let now = utc(2026, 10, 1, 0, 0);
        assert_eq!(ReminderRequest::parse("", sgt(), now), Ok(None));
        assert_eq!(ReminderRequest::parse("no date here", sgt(), now), Ok(None));
        assert_eq!(
            ReminderRequest::parse("x 31/02/2027 08:00", sgt(), now),
            Err(InputError::InvalidDateTime)
        );
        assert_eq!(
            ReminderRequest::parse("x 01/01/2027 25:00", sgt(), now),
            Err(InputError::InvalidDateTime)
        );
        assert_eq!(
            ReminderRequest::parse("x 01/01/2027 08:00 yearly", sgt(), now),
            Err(InputError::InvalidRecurrence)
        );
        assert_eq!(
            ReminderRequest::parse("x 01/01/2020 08:00", sgt(), now),
            Err(InputError::InPast)
        );
    }

    #[test]
    fn past_recurring_reminder_moves_to_next_occurrence() {
        let now = utc(2026, 10, 1, 3, 0); // 11:00 in UTC+8
        let request = ReminderRequest::parse("standup 01/10/2026 10:00 daily", sgt(), now)
            .unwrap()
            .unwrap();
        assert_eq!(request.due_at, utc(2026, 10, 2, 2, 0));
    }

    fn sender() -> Sender {
        Sender {
            chat_id: 7,
            first_name: "Ada".into(),
            last_name: String::new(),
        }
    }

    fn bot() -> BotInfo {
        BotInfo {
            name: "Naggy".into(),
            username: "naggybot".into(),
        }
    }

    #[tokio::test]
    async fn commands_require_registration() {
        let db = Db::open_in_memory().unwrap();
        let reply = execute(Command::ListReminders, &sender(), &bot(), &db)
            .await
            .unwrap();
        assert!(reply.contains("please register"), "{reply}");
    }

    #[tokio::test]
    async fn end_to_end_command_flow() {
        let db = Db::open_in_memory().unwrap();
        let (s, b) = (sender(), bot());

        let reply = execute(Command::Register("+8".into()), &s, &b, &db)
            .await
            .unwrap();
        assert!(reply.contains("UTC+08:00"), "{reply}");

        let reply = execute(
            Command::RemindMe("stretch 01/01/2099 08:00 weekly".into()),
            &s,
            &b,
            &db,
        )
        .await
        .unwrap();
        assert!(
            reply.contains("01/01/2099 08:00 (UTC+08:00), weekly"),
            "{reply}"
        );

        let reply = execute(Command::ListReminders, &s, &b, &db).await.unwrap();
        assert!(
            reply.contains("#1 stretch — 01/01/2099 08:00 (weekly)"),
            "{reply}"
        );

        let reply = execute(Command::RemoveReminder("abc".into()), &s, &b, &db)
            .await
            .unwrap();
        assert!(reply.starts_with("Usage"), "{reply}");

        let reply = execute(Command::RemoveReminder("#1".into()), &s, &b, &db)
            .await
            .unwrap();
        assert!(reply.contains("removed reminder #1"), "{reply}");

        let reply = execute(Command::Unregister, &s, &b, &db).await.unwrap();
        assert!(reply.starts_with("Goodbye"), "{reply}");
    }
}
