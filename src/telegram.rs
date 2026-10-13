//! Telegram glue: turns updates into conversation calls and replies into API requests.

use std::sync::Arc;

use chrono::Utc;
use teloxide::adaptors::Throttle;
use teloxide::prelude::*;
use teloxide::types::{
    ButtonRequest, ChatKind, KeyboardButton, KeyboardMarkup, KeyboardRemove, ReplyMarkup,
};
use teloxide::utils::command::{BotCommands, ParseError};
use teloxide::{ApiError, RequestError};

use crate::commands::{self, BotInfo, Command};
use crate::conversation::{self, Markup, Reply, Sender};
use crate::db::Db;
use crate::model::Nagging;
use crate::panels;

pub type Bot = Throttle<teloxide::Bot>;
pub type HandlerResult = anyhow::Result<()>;

pub struct AppState {
    pub db: Db,
    pub bot_info: BotInfo,
}

const INTERNAL_ERROR: &str = "Sorry, something went wrong. Please try again later.";

pub async fn handle_message(bot: Bot, msg: Message, state: Arc<AppState>) -> HandlerResult {
    let sender = sender_of(&msg);
    let now = Utc::now();
    let db = &state.db;

    let result = if let Some(location) = msg.location() {
        conversation::on_location(db, &sender, location.latitude, location.longitude, now).await
    } else if let Some(text) = msg.text().or(msg.caption()) {
        let forwarded = msg.forward_origin().is_some();
        if text.starts_with('/') && !forwarded {
            match Command::parse(text, &state.bot_info.username) {
                Ok(command) => commands::execute(command, &sender, &state.bot_info, db, now).await,
                // Addressed to another bot in a group chat.
                Err(ParseError::WrongBotName(_)) => return Ok(()),
                Err(_) => Ok(vec![Reply::text(
                    "Sorry, I don't know that command. Send /help to see what I can do.",
                )]),
            }
        } else if sender.private {
            conversation::on_text(db, &sender, text, forwarded, now).await
        } else {
            // In groups, only commands are handled.
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let replies = result.unwrap_or_else(|error| {
        tracing::error!(?error, chat_id = sender.chat_id, "failed to handle message");
        vec![Reply::text(INTERNAL_ERROR)]
    });
    for reply in replies {
        send(&bot, msg.chat.id, reply).await?;
    }
    Ok(())
}

pub async fn handle_callback(
    bot: Bot,
    query: CallbackQuery,
    state: Arc<AppState>,
) -> HandlerResult {
    let message = query.regular_message();
    let user_id = message.map_or(query.from.id.0 as i64, |m| m.chat.id.0);
    let data = query.data.as_deref().unwrap_or_default();

    let reply = match conversation::on_callback(
        &state.db,
        user_id,
        data,
        message.and_then(|m| m.text()),
        Utc::now(),
    )
    .await
    {
        Ok(reply) => reply,
        Err(error) => {
            tracing::error!(?error, user_id, data, "failed to handle button");
            bot.answer_callback_query(query.id.clone())
                .text(INTERNAL_ERROR)
                .await?;
            return Ok(());
        }
    };

    let mut answer = bot.answer_callback_query(query.id.clone());
    if let Some(toast) = reply.toast {
        answer = answer.text(toast);
    }
    answer.await?;

    if let (Some(message), Some(panel)) = (message, reply.edit) {
        let unchanged = message.text() == Some(panel.text.as_str())
            && message.reply_markup().map(|m| &m.inline_keyboard)
                == Some(&panel.keyboard.inline_keyboard);
        if !unchanged
            && let Err(error) = bot
                .edit_message_text(message.chat.id, message.id, panel.text)
                .reply_markup(panel.keyboard)
                .await
        {
            tracing::debug!(?error, "could not edit message");
        }
    }
    Ok(())
}

async fn send(bot: &Bot, chat_id: ChatId, reply: Reply) -> Result<(), RequestError> {
    let request = bot.send_message(chat_id, reply.text);
    let request = match reply.markup {
        Markup::None => request,
        Markup::Inline(keyboard) => request.reply_markup(keyboard),
        Markup::AskLocation => request.reply_markup(ReplyMarkup::Keyboard(
            KeyboardMarkup::new([[
                KeyboardButton::new("📍 Share my location").request(ButtonRequest::Location)
            ]])
            .resize_keyboard()
            .one_time_keyboard(),
        )),
        Markup::RemoveKeyboard => {
            request.reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
        }
    };
    request.await?;
    Ok(())
}

/// Sends a reminder with Done/Snooze buttons. Returns `Ok(false)` if the chat can no longer
/// be reached (bot blocked, chat deleted, ...), so the caller can stop nagging it.
pub async fn send_nagging(bot: &Bot, nagging: &Nagging) -> Result<bool, RequestError> {
    match bot
        .send_message(
            ChatId(nagging.user_id),
            format!("⏰ {}", nagging.description),
        )
        .reply_markup(panels::nag_keyboard(nagging.id))
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
        private: matches!(msg.chat.kind, ChatKind::Private(_)),
    }
}

/// Publishes the command menu shown by Telegram clients.
pub async fn publish_commands(bot: &Bot) {
    if let Err(error) = bot.set_my_commands(Command::bot_commands()).await {
        tracing::warn!(?error, "failed to publish the command list");
    }
}
