//! Slash commands. Most things are done with buttons; commands are entry points.

use std::sync::LazyLock;

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use teloxide::utils::command::BotCommands;

use crate::conversation::{self, Reply, Sender};
use crate::db::Db;
use crate::model::Recurrence;
use crate::panels;

const SOURCE_URL: &str = "https://dev.decryptology.net/decryp7/ExtremelyNaggyBot";

#[derive(BotCommands, Clone, Debug, PartialEq, Eq)]
#[command(rename_rule = "lowercase", description = "Commands:")]
pub enum Command {
    #[command(hide)]
    Start,
    #[command(description = "how to use this bot.")]
    Help,
    #[command(description = "add a reminder, e.g. /remindme call mum.")]
    RemindMe(String),
    #[command(description = "see, change or delete your reminders.", aliases = ["listreminders"], hide_aliases)]
    List,
    #[command(description = "show or change your timezone, e.g. /timezone London.", aliases = ["register"], hide_aliases)]
    Timezone(String),
    #[command(description = "delete all your reminders.", aliases = ["removeallreminders"], hide_aliases)]
    ClearAll,
    #[command(description = "delete all your data.")]
    Unregister,
    #[command(description = "about this bot.")]
    About,
    /// Kept for users of the old text interface; reminders are now deleted from /list.
    #[command(hide)]
    RemoveReminder(String),
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
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Reply>> {
    let name = &sender.first_name;
    let user = db.user(sender.chat_id).await?;

    Ok(match command {
        Command::Start => match user {
            Some(_) => vec![Reply::text(format!(
                "Hello {name}, {} at your service.\n\n{}",
                bot.name,
                usage(sender)
            ))],
            None => vec![conversation::ask_timezone(
                sender,
                &format!(
                    "Hello {name}, I'm {}. I remind you about things and keep nagging every minute until you tap Done.\n\nTo get started, which timezone are you in?",
                    bot.name
                ),
            )],
        },
        Command::Help => vec![Reply::text(format!(
            "{}\n\n{}",
            usage(sender),
            Command::descriptions().username(&bot.username)
        ))],
        Command::About => vec![Reply::text(format!(
            "I'm an extremely naggy reminder bot: once a reminder is due, I repeat it every minute until you tap Done.\n\nBug reports: @decryp7\nSource code: {SOURCE_URL}"
        ))],

        Command::RemindMe(text) if text.trim().is_empty() => vec![Reply::text(
            "What should I remind you about? For example: /remindme call mum",
        )],
        Command::RemindMe(text) => match (&user, legacy_reminder(&text)) {
            (Some(user), Some((description, at, recurrence))) => {
                let mut due = user.tz.utc(at);
                if due <= now {
                    match recurrence.next_after(due, now, user.tz) {
                        Some(next) => due = next,
                        None => return Ok(vec![Reply::text("That time is already in the past.")]),
                    }
                }
                let reminder = db
                    .add_reminder(user.id, description, due, recurrence)
                    .await?;
                vec![panels::saved(&reminder, user, now, "✅ Reminder saved").into()]
            }
            _ => conversation::start_reminder_for(db, sender, &text, now).await?,
        },

        Command::List => match user {
            Some(user) => {
                let reminders = db.reminders_for_user(user.id).await?;
                vec![panels::list(&reminders, &user).into()]
            }
            None => vec![not_registered(sender)],
        },

        Command::Timezone(arg) if arg.trim().is_empty() => {
            let intro = match &user {
                Some(user) => format!("Your timezone is {}. To change it:", user.tz),
                None => "Which timezone are you in?".into(),
            };
            vec![conversation::ask_timezone(sender, &intro)]
        }
        Command::Timezone(arg) => match crate::model::UserTz::parse(&arg) {
            Ok(tz) => conversation::set_timezone(db, sender, tz, now).await?,
            Err(error) => vec![conversation::ask_timezone(sender, &error.to_string())],
        },

        Command::ClearAll => match db.delete_all_reminders(sender.chat_id).await? {
            0 => vec![Reply::text("You have no reminders.")],
            1 => vec![Reply::text("I have deleted your reminder.")],
            n => vec![Reply::text(format!(
                "I have deleted all {n} of your reminders."
            ))],
        },
        Command::Unregister => {
            let removed = db.delete_user(sender.chat_id).await?;
            vec![Reply::text(if removed {
                format!("Goodbye {name}, I have deleted you and all your reminders.")
            } else {
                "You are not registered.".to_owned()
            })]
        }

        Command::RemoveReminder(_) => vec![Reply::text(
            "Use /list and tap a reminder to change or delete it.",
        )],
    })
}

/// The one-line format of earlier versions:
/// `/remindme <description> <dd/mm/yyyy> <HH:MM> [once|daily|weekly|monthly]`.
fn legacy_reminder(args: &str) -> Option<(String, NaiveDateTime, Recurrence)> {
    static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?is)^\s*(?P<description>.+?)\s+(?P<at>\d{1,2}/\d{1,2}/\d{4}\s+\d{1,2}:\d{2})(?:\s+(?P<recurrence>once|daily|weekly|monthly))?\s*$",
        )
        .expect("valid regex")
    });
    let caps = PATTERN.captures(args)?;
    let at = NaiveDateTime::parse_from_str(
        &caps["at"].split_whitespace().collect::<Vec<_>>().join(" "),
        "%d/%m/%Y %H:%M",
    )
    .ok()?;
    let recurrence = caps
        .name("recurrence")
        .map_or(Some(Recurrence::Once), |m| m.as_str().parse().ok())?;
    Some((caps["description"].trim().to_owned(), at, recurrence))
}

fn usage(sender: &Sender) -> String {
    if sender.private {
        "Send me any message, e.g. \"call mum\", and pick when with the buttons. Use /list to change or delete reminders.".into()
    } else {
        "Use /remindme followed by what to remind you about, e.g. /remindme team lunch, and pick when with the buttons. Use /list to change or delete reminders.".into()
    }
}

fn not_registered(sender: &Sender) -> Reply {
    conversation::ask_timezone(
        sender,
        "You don't have any reminders yet. First, which timezone are you in?",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands_and_old_names() {
        assert_eq!(
            Command::parse("/start", "naggybot").unwrap(),
            Command::Start
        );
        assert_eq!(
            Command::parse("/list@naggybot", "naggybot").unwrap(),
            Command::List
        );
        assert_eq!(
            Command::parse("/listreminders", "naggybot").unwrap(),
            Command::List
        );
        assert_eq!(
            Command::parse("/register +8", "naggybot").unwrap(),
            Command::Timezone("+8".into())
        );
        assert_eq!(
            Command::parse("/remindme pay rent", "naggybot").unwrap(),
            Command::RemindMe("pay rent".into())
        );
        assert_eq!(
            Command::parse("/removeallreminders", "naggybot").unwrap(),
            Command::ClearAll
        );
        assert!(Command::parse("/unknown", "naggybot").is_err());
    }

    #[test]
    fn parses_legacy_one_line_reminders() {
        let (description, at, recurrence) =
            legacy_reminder("Take my pills 25/12/2026 08:00 Daily").unwrap();
        assert_eq!(description, "Take my pills");
        assert_eq!(at.to_string(), "2026-12-25 08:00:00");
        assert_eq!(recurrence, Recurrence::Daily);
        assert_eq!(
            legacy_reminder("dentist 2/11/2026 9:30").unwrap().2,
            Recurrence::Once
        );
        assert!(legacy_reminder("call mum").is_none());
        assert!(legacy_reminder("x 31/02/2027 08:00").is_none());
    }

    #[test]
    fn help_hides_old_names() {
        let help = Command::descriptions().to_string();
        assert!(help.contains("/list"));
        assert!(!help.contains("listreminders"));
        assert!(!help.contains("removereminder"));
    }
}
