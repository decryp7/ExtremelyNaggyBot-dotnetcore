//! Telegram glue: update handlers and outgoing reminder messages.

use std::sync::Arc;

use serde::Deserialize;
use teloxide::adaptors::Throttle;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::{BotCommands, ParseError};
use teloxide::{ApiError, RequestError};

use crate::commands::{self, BotInfo, Command, Sender};
use crate::db::Db;
use crate::model::Nagging;

pub type Bot = Throttle<teloxide::Bot>;
pub type HandlerResult = anyhow::Result<()>;

pub struct AppState {
    pub db: Db,
    pub bot_info: BotInfo,
}

const ACK_PREFIX: &str = "ack:";

pub async fn handle_message(bot: Bot, msg: Message, state: Arc<AppState>) -> HandlerResult {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let is_private = matches!(msg.chat.kind, ChatKind::Private(_));
    if !text.starts_with('/') {
        if is_private {
            bot.send_message(
                msg.chat.id,
                "I only understand commands. Send /help to see them.",
            )
            .await?;
        }
        return Ok(());
    }

    let command = match Command::parse(text, &state.bot_info.username) {
        Ok(command) => command,
        // Addressed to another bot in a group chat.
        Err(ParseError::WrongBotName(_)) => return Ok(()),
        Err(_) => {
            let help = Command::descriptions()
                .username(&state.bot_info.username)
                .to_string();
            bot.send_message(
                msg.chat.id,
                format!("Sorry, I do not know that command.\n\n{help}"),
            )
            .await?;
            return Ok(());
        }
    };

    let sender = sender_of(&msg);
    let reply = match commands::execute(command.clone(), &sender, &state.bot_info, &state.db).await
    {
        Ok(reply) => reply,
        Err(error) => {
            tracing::error!(?error, ?command, chat_id = sender.chat_id, "command failed");
            "Sorry, something went wrong. Please try again later.".to_owned()
        }
    };
    bot.send_message(msg.chat.id, reply).await?;
    Ok(())
}

pub async fn handle_callback(
    bot: Bot,
    query: CallbackQuery,
    state: Arc<AppState>,
) -> HandlerResult {
    let message = query.regular_message();
    let user_id = message.map_or(query.from.id.0 as i64, |m| m.chat.id.0);

    let acknowledged = match query.data.as_deref().and_then(Acknowledgement::parse) {
        Some(Acknowledgement::Nagging(id)) => state.db.acknowledge_nagging(user_id, id).await?,
        Some(Acknowledgement::LegacyReminder(id)) => {
            state.db.acknowledge_reminder(user_id, id).await?
        }
        None => false,
    };

    let text = if acknowledged {
        "Acknowledged ✅"
    } else {
        "Already acknowledged"
    };
    bot.answer_callback_query(query.id.clone())
        .text(text)
        .await?;

    // Remove the button so it is clear the reminder has been dealt with.
    if let Some(message) = message
        && let Err(error) = bot
            .edit_message_reply_markup(message.chat.id, message.id)
            .await
    {
        tracing::debug!(?error, "could not remove acknowledge button");
    }
    Ok(())
}

/// Sends a reminder with an "Acknowledge" button. Returns `Ok(false)` if the chat can no
/// longer be reached (bot blocked, chat deleted, ...), so the caller can stop nagging it.
pub async fn send_nagging(bot: &Bot, nagging: &Nagging) -> Result<bool, RequestError> {
    let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "Acknowledge",
        format!("{ACK_PREFIX}{}", nagging.id),
    )]]);
    match bot
        .send_message(
            ChatId(nagging.user_id),
            format!("⏰ {}", nagging.description),
        )
        .reply_markup(keyboard)
        .await
    {
        Ok(_) => Ok(true),
        Err(RequestError::Api(error)) if is_unreachable(&error) => Ok(false),
        Err(error) => Err(error),
    }
}

fn is_unreachable(error: &ApiError) -> bool {
    matches!(
        error,
        ApiError::BotBlocked
            | ApiError::BotKicked
            | ApiError::BotKickedFromSupergroup
            | ApiError::ChatNotFound
            | ApiError::UserDeactivated
            | ApiError::CantInitiateConversation
    )
}

fn sender_of(msg: &Message) -> Sender {
    let (first_name, last_name) = match &msg.from {
        Some(user) => (
            user.first_name.clone(),
            user.last_name.clone().unwrap_or_default(),
        ),
        None => (
            msg.chat.title().unwrap_or("there").to_owned(),
            String::new(),
        ),
    };
    Sender {
        chat_id: msg.chat.id.0,
        first_name,
        last_name,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Acknowledgement {
    Nagging(i64),
    /// `{"Id": <reminder_id>}`, the callback data of buttons sent by the .NET bot.
    LegacyReminder(i64),
}

impl Acknowledgement {
    fn parse(data: &str) -> Option<Self> {
        if let Some(id) = data.strip_prefix(ACK_PREFIX) {
            return id.parse().ok().map(Self::Nagging);
        }

        #[derive(Deserialize)]
        struct Legacy {
            #[serde(rename = "Id")]
            id: i64,
        }
        serde_json::from_str::<Legacy>(data)
            .ok()
            .map(|legacy| Self::LegacyReminder(legacy.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_acknowledgements() {
        assert_eq!(
            Acknowledgement::parse("ack:42"),
            Some(Acknowledgement::Nagging(42))
        );
        assert_eq!(
            Acknowledgement::parse(r#"{"Id":7}"#),
            Some(Acknowledgement::LegacyReminder(7))
        );
        assert_eq!(Acknowledgement::parse("ack:x"), None);
        assert_eq!(Acknowledgement::parse("garbage"), None);
    }
}
