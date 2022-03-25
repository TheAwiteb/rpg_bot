// rpg_bot - Telegram bot 🤖, help you to run and share Rust code in Telegram via Rust playground 🦀
// Source code: <https://github.com/TheAwiteb/rpg_bot>
//
// Copyright (C) 2022 TheAwiteb <awiteb@hotmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::models::Users;
use crate::{
    keyboards,
    models::{Config, NewSourceCode, SourceCode},
    rpg,
    rpg_db::{self, languages_ctx},
};
use chrono::offset;
use diesel::SqliteConnection;
use futures::try_join;
use json_gettext::get_text;
use std::collections::HashMap;
use std::error::Error;
use strfmt::strfmt;
use teloxide::utils::command::parse_command;
use teloxide::{
    prelude2::*,
    types::{ForwardedFrom, InlineKeyboardMarkup, ParseMode, User},
    utils::command::BotCommand,
    RequestError,
};

#[derive(BotCommand)]
pub enum Command {
    #[command(parse_with = "split")]
    Help,
    #[command(parse_with = "split")]
    Run {
        version: String,
        mode: String,
        edition: String,
    },
    #[command(parse_with = "split")]
    Share {
        version: String,
        mode: String,
        edition: String,
    },
}

impl Command {
    fn args(&self) -> Option<(&str, &str, &str)> {
        match self {
            Command::Run {
                version,
                mode,
                edition,
            } => Some((version, mode, edition)),
            Command::Share {
                version,
                mode,
                edition,
            } => Some((version, mode, edition)),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        match self {
            #[allow(unused_variables)]
            Command::Share {
                version,
                mode,
                edition,
            } => "share",

            #[allow(unused_variables)]
            Command::Run {
                version,
                mode,
                edition,
            } => "run",
            Command::Help => "help",
        }
    }
}

impl From<(&NewSourceCode, &str)> for Command {
    fn from((code, command_name): (&NewSourceCode, &str)) -> Command {
        if command_name.to_ascii_lowercase() == *"run" {
            Command::Run {
                version: code.version.clone(),
                mode: code.mode.clone(),
                edition: code.edition.clone(),
            }
        } else {
            Command::Share {
                version: code.version.clone(),
                mode: code.mode.clone(),
                edition: code.edition.clone(),
            }
        }
    }
}

/// Returns bot username
pub async fn bot_username(bot: &AutoSend<Bot>) -> String {
    bot.get_me()
        .await
        .unwrap()
        .user
        .username
        .expect("Bots must have usernames")
}

/// Returns wait message of command (Run and Share) else return `None`
fn get_wait_message(command: &Command, language: &str) -> Option<String> {
    if let Some((version, mode, edition)) = command.args() {
        let mut vars: HashMap<String, String> = HashMap::new();
        let ctx = languages_ctx();
        vars.insert("version".into(), version.into());
        vars.insert("mode".into(), mode.into());
        vars.insert("edition".into(), edition.into());
        Some(
            strfmt(
                &get_text!(
                    ctx,
                    language,
                    if command.name() == "run" {
                        "RUN_MESSAGE"
                    } else {
                        "SHARE_MESSAGE"
                    }
                )
                .unwrap()
                .to_string(),
                &vars,
            )
            .unwrap(),
        )
    } else {
        None
    }
}

async fn already_use_answer(
    requester: &AutoSend<Bot>,
    query_id: &str,
    language: &str,
    is_run: bool,
) {
    let ctx = languages_ctx();
    requester
        .answer_callback_query(query_id)
        .text(
            get_text!(
                ctx,
                language,
                if is_run {
                    "ALREADY_RUN"
                } else {
                    "ALREADY_SHARE"
                }
            )
            .unwrap()
            .to_string(),
        )
        .send()
        .await
        .log_on_error()
        .await;
}

async fn replay_wait_message(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    message_id: i32,
    command: &Command,
    language: &str,
) -> Result<Message, Box<dyn Error + Send + Sync>> {
    Ok(bot
        .send_message(chat_id, get_wait_message(command, language).unwrap())
        .reply_to_message_id(message_id)
        .send()
        .await?)
}

async fn send_wait_message(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    command: &Command,
    language: &str,
) -> Result<Message, Box<dyn Error + Send + Sync>> {
    Ok(bot
        .send_message(chat_id, get_wait_message(command, language).unwrap())
        .send()
        .await?)
}

fn delay_error_message(author: &Users, is_command: bool, conn: &mut SqliteConnection) -> String {
    let mut vars: HashMap<String, String> = HashMap::new();
    let ctx = languages_ctx();

    vars.insert(
        "delay".to_string(),
        ((if is_command {
            author.last_command_record
        } else {
            author.last_button_record
        }
        .unwrap() // The use of unwrap here is normal, because if no record is made to the user, the
        // `can_send_command` and `can_click_button` functions will return `true`.
        .timestamp()
            + if is_command {
                // default value is 15
                Config::get_or_add("command_delay", "15", conn)
                    .value
                    .parse::<i64>()
                    .expect("`command_delay` config should be integer")
            } else {
                // default value is 2
                Config::get_or_add("button_delay", "2", conn)
                    .value
                    .parse::<i64>()
                    .expect("`button_delay` config should be integer")
            })
            - (offset::Utc::now().timestamp()))
        .to_string(),
    );
    strfmt(
        &get_text!(
            ctx,
            &author.language,
            if is_command {
                "SPAM_COMMAND_MESSAGE"
            } else {
                "SPAM_CLICK_MESSAGE"
            }
        )
        .unwrap()
        .to_string(),
        &vars,
    )
    .unwrap()
}

fn attempt_error_message(author: &Users) -> String {
    let mut vars: HashMap<String, String> = HashMap::new();
    let ctx = languages_ctx();

    vars.insert(
        "attempts_maximum".into(),
        author.attempts_maximum.to_string(),
    );
    strfmt(
        &get_text!(ctx, &author.language, "EXCEEDED_ATTEMPTS_MESSAGE")
            .unwrap()
            .to_string(),
        &vars,
    )
    .unwrap()
}

/// Share and run, and make attempt for user
async fn share_run_answer(
    bot: &AutoSend<Bot>,
    command: &Command,
    already_use_keyboard: bool,
    message: &Message,
    author: &mut Users,
    code: &NewSourceCode,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let output: Result<String, String> = if command.name() == "run" {
        rpg::run(&code.into()).await
    } else {
        rpg::share(&code.into()).await
    };

    if output.is_ok() {
        code.save(conn)?;
    };

    let (keyboard, output): (InlineKeyboardMarkup, String) = if command.name() == "run" {
        (
            keyboards::view_share_keyboard(
                code.code.clone(),
                already_use_keyboard,
                output.is_ok(),
                &author.language,
            ),
            match output {
                Ok(output) => output,
                Err(output) => output,
            },
        )
    } else {
        (
            keyboards::view_run_keyboard(
                code.code.clone(),
                already_use_keyboard,
                output.is_ok(),
                &author.language,
            ),
            match output {
                Ok(output) => output,
                Err(output) => output,
            },
        )
    };
    author.make_attempt(conn).log_on_error().await;
    bot.edit_message_text(
        // For text messages, the actual UTF-8 text of the message, 0-4096 characters
        // https://core.telegram.org/bots/api#message
        message.chat.id,
        message.id,
        output
            .chars()
            .take(if output.chars().count() > 4096 {
                4096
            } else {
                output.chars().count()
            })
            .collect::<String>(),
    )
    .reply_markup(keyboard)
    .send()
    .await
    .log_on_error()
    .await;

    Ok(())
}

/// Send code output for run command and Rust playground for share command
pub async fn share_run_answer_message(
    bot: &AutoSend<Bot>,
    message: &Message,
    command: &Command,
    author: &Users,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    SourceCode::filter_source_codes(conn).unwrap();
    let source_code_message: &Message = message.reply_to_message().unwrap();
    if let Some(source_code) = source_code_message.text() {
        if let Some((version, mode, edition)) = command.args() {
            let code: rpg::Code = rpg::Code::new(source_code, version, mode, edition);
            if let Err(err) = code.is_valid() {
                bot.send_message(message.chat.id, err)
                    .reply_to_message_id(message.id)
                    .send()
                    .await
                    .log_on_error()
                    .await;
            } else {
                let reply_message: Message = replay_wait_message(
                    bot,
                    message.chat.id,
                    message.id,
                    command,
                    &author.language,
                )
                .await?;
                share_run_answer(
                    bot,
                    command,
                    false,
                    &reply_message,
                    &mut rpg_db::get_user(conn, message.from().unwrap()).unwrap(),
                    &NewSourceCode::new(conn, &code, author)?,
                    conn,
                )
                .await
                .log_on_error()
                .await;
            }
        }
    } else {
        let ctx = languages_ctx();
        bot.send_message(
            message.chat.id,
            get_text!(ctx, &author.language, "MUST_BE_TEXT")
                .unwrap()
                .to_string(),
        )
        .reply_to_message_id(message.id)
        .send()
        .await
        .log_on_error()
        .await;
    }
    Ok(())
}

pub async fn share_run_answer_callback(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    command: &str,
    code: NewSourceCode,
    author: &User,
    language: &str,
    conn: &mut SqliteConnection,
) -> Result<(), RequestError> {
    let message: Message =
        send_wait_message(bot, chat_id, &Command::from((&code, command)), language)
            .await
            .unwrap();
    share_run_answer(
        bot,
        &Command::from((&code, command)),
        true,
        &message,
        &mut rpg_db::get_user(conn, author).unwrap(),
        &code,
        conn,
    )
    .await
    .log_on_error()
    .await;
    Ok(())
}

/// Returns the code args from command args
/// Example:
/// ```rust
/// assert_eq!(get_args(Vec::new()), vec!["stable".to_string(), "debug".to_string(), "2021".to_string()])
/// assert_eq!(get_args(vec!["beta"]), vec!["beta".to_string(), "debug".to_string(), "2021".to_string()])
/// ````
fn get_args(args: Vec<&str>) -> Vec<String> {
    let default: Vec<String> = vec!["stable".into(), "debug".into(), "2021".into()];
    match args.len() {
        0 => default,
        1 => vec![args[0].to_ascii_lowercase(), "debug".into(), "2021".into()],
        2 => vec![
            args[0].to_ascii_lowercase(),
            args[1].to_ascii_lowercase(),
            "2021".into(),
        ],
        _ => vec![
            args[0].to_ascii_lowercase(),
            args[1].to_ascii_lowercase(),
            args[2].to_ascii_lowercase(),
        ],
    }
}

/// returns None that mean the message is deleted else message content
fn get_source_code(code: &str, conn: &mut SqliteConnection) -> Option<SourceCode> {
    SourceCode::get_by_code(code, conn).ok()
}

async fn cannot_reached_answer(bot: &AutoSend<Bot>, query_id: &str, language: &str) {
    let ctx = languages_ctx();
    bot.answer_callback_query(query_id)
        .text(
            get_text!(ctx, language, "SOURCES_CANNOT_REACHED")
                .unwrap()
                .to_string(),
        )
        .send()
        .await
        .log_on_error()
        .await;
}

async fn run_share_callback(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    command: &str,
    code: String,
    language: &str,
    conn: &mut SqliteConnection,
) {
    // share and run commands need source code
    // if get_source_code returns None that mean the source code message is deleted
    if let Some(source_code) = get_source_code(&code, conn) {
        let message: Message = callback_query.clone().message.unwrap();
        let keyboard: InlineKeyboardMarkup = if command == "share" {
            keyboards::view_share_keyboard(code, true, true, language)
        } else {
            keyboards::view_run_keyboard(code, true, true, language)
        };
        try_join!(
            share_run_answer_callback(
                bot,
                message.chat.id,
                command,
                source_code.into(),
                &callback_query.from,
                language,
                conn
            ),
            bot.edit_message_reply_markup(message.chat.id, message.id)
                .reply_markup(keyboard)
                .send()
        )
        .log_on_error()
        .await;
    } else {
        cannot_reached_answer(bot, &callback_query.id, language).await;
    }
}

async fn update_options(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    code: &str,
    option_name: &str,
    option_value: &str,
    language: &str,
    conn: &mut SqliteConnection,
) {
    if let Ok(mut source) = SourceCode::get_by_code(code, conn) {
        let mut vars: HashMap<String, String> = HashMap::new();
        let ctx = languages_ctx();

        vars.insert(
            "option_name".into(),
            get_text!(ctx, language, option_name.to_ascii_uppercase())
                .unwrap()
                .to_string(),
        );
        vars.insert("option_value".into(), option_value.to_string());

        let message: Message = callback_query.clone().message.unwrap();
        let old_keyboard: &InlineKeyboardMarkup = message.reply_markup().unwrap();
        let change_option_name_answer = bot
            .answer_callback_query(&callback_query.id)
            .text(
                strfmt(
                    &get_text!(ctx, language, "SET_MESSAGE").unwrap().to_string(),
                    &vars,
                )
                .unwrap(),
            )
            .send();

        source
            .update_by_name(option_name, option_value, conn)
            .log_on_error()
            .await;

        let keyboard: InlineKeyboardMarkup =
            if old_keyboard.inline_keyboard[4][0].text.contains("Run") {
                keyboards::run_keyboard(source, language)
            } else {
                keyboards::share_keyboard(source, language)
            };

        if &keyboard != old_keyboard {
            try_join!(
                change_option_name_answer,
                bot.edit_message_reply_markup(message.chat.id, message.id)
                    .reply_markup(keyboard)
                    .send()
            )
            .log_on_error()
            .await;
        } else {
            change_option_name_answer.await.log_on_error().await;
        }
    } else {
        cannot_reached_answer(bot, &callback_query.id, language).await;
    }
}

/// Answer to change language request (CallbackQuery and Message)
async fn change_language(
    bot: &AutoSend<Bot>,
    author: &mut Users,
    message_id: i32,
    chat_id: i64,
    new_language: &str,
    query_id: &str,
    conn: &mut SqliteConnection,
) {
    let ctx = languages_ctx();
    let new_language = new_language.replace('_', " ");

    // if new language same old one
    if new_language == author.language {
        bot.answer_callback_query(query_id)
            .text(
                get_text!(ctx, &new_language, "ALREADY_CURRENT_LANGUAGE")
                    .unwrap()
                    .to_string()
                    + " 🤨",
            )
            .send()
            .await
            .log_on_error()
            .await;
    } else {
        author
            .update_language(&new_language, conn)
            .log_on_error()
            .await;
        bot.edit_message_text(
            chat_id,
            message_id,
            get_text!(ctx, &new_language, "CHANGE_LANGUAGE_SUCCESSFULLY")
                .unwrap()
                .to_string()
                + " 🤖",
        )
        .reply_markup(keyboards::add_lang_keyboard(&new_language))
        .send()
        .await
        .log_on_error()
        .await;
    }
}

/// Returns information about `author` with `language` language
pub fn info_text(author: &Users, language: &str, conn: &mut SqliteConnection) -> String {
    let ctx = languages_ctx();
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert(
        "username".into(),
        match &author.username {
            Some(username) => format!("@{}", username),
            None => get_text!(ctx, language, "NOT_FOUND").unwrap().to_string(),
        },
    );
    vars.insert("telegram_id".into(), author.telegram_id.clone());
    vars.insert("lang".into(), author.language.clone());
    vars.insert(
        "admin".into(),
        if author.is_admin { "✔️" } else { "✖️" }.to_string(),
    );
    vars.insert(
        "ban".into(),
        if author.is_ban { "✔️" } else { "✖️" }.to_string(),
    );
    vars.insert(
        "ban_date".into(),
        match author.ban_date {
            Some(date) => date.to_string(),
            None => get_text!(ctx, language, "NOT_FOUND").unwrap().to_string(),
        },
    );
    vars.insert("full_name".into(), author.telegram_fullname.clone());
    vars.insert(
        "command_delay".into(),
        // default value is 15
        Config::get_or_add("command_delay", "15", conn).value,
    );
    vars.insert(
        "button_delay".to_string(),
        // default value is 2
        Config::get_or_add("button_delay", "2", conn).value,
    );
    vars.insert(
        "attempts_maximum".into(),
        author.attempts_maximum.to_string(),
    );
    vars.insert("attempts".into(), author.attempts.to_string());
    vars.insert(
        "attempts_have".into(),
        (author.attempts_maximum - author.attempts).to_string(),
    );

    strfmt(
        &get_text!(ctx, language, "INFO_MESSAGE")
            .unwrap()
            .to_string(),
        &vars,
    )
    .unwrap()
}

/// admin/unadmin user by telegram id
/// > Note: All `String` is less than 64 bytes
fn admin_unadmin(user_id: i64, author: &Users, conn: &mut SqliteConnection) -> String {
    let ctx = languages_ctx();
    if user_id.eq(&author.telegram_id.parse::<i64>().unwrap()) {
        get_text!(ctx, &author.language, "CANNOT_UNADMIN_YORSELF")
            .unwrap()
            .to_string()
    } else if let Some(mut user) = Users::get_by_telegram_id(conn, user_id.to_string()) {
        if user.is_admin && author.telegram_id.ne(&rpg_db::super_user_id().to_string()) {
            get_text!(ctx, &author.language, "CANNOT_UNADMIN_ADMIN")
                .unwrap()
                .to_string()
        } else if user.switch_admin_stutes(conn).is_err() {
            // If there error
            get_text!(ctx, &author.language, "ERROR_WHILE_DO")
                .unwrap()
                .to_string()
        } else if user.is_ban {
            get_text!(ctx, &author.language, "CANNOT_ADMIN_BANNED_USER")
                .unwrap()
                .to_string()
        } else if user.is_admin {
            // This means it was admin
            get_text!(ctx, &author.language, "SUCCESSFULLY_ADMIN")
                .unwrap()
                .to_string()
        } else {
            // This means it wasn't admin
            get_text!(ctx, &author.language, "SUCCESSFULLY_UNADMIN")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "USER_NOT_FOUND")
            .unwrap()
            .to_string()
    }
}

/// admin/unadmin user by username
/// > Note: All `String` is less than 64 bytes
fn admin_unadmin_by_username(
    username: &str,
    author: &Users,
    conn: &mut SqliteConnection,
) -> String {
    let ctx = languages_ctx();
    if author.is_admin {
        if let Some(user) = Users::get_by_telegram_username(conn, username) {
            // `0` here will appear user not found message (:
            admin_unadmin(user.telegram_id.parse().unwrap_or(0), author, conn)
        } else {
            get_text!(ctx, &author.language, "INVALID_ID_ERROR")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "ADMIN_COMMAND_ERROR")
            .unwrap()
            .to_string()
    }
}

/// admin/unadmin user by message
/// > Note: All `String` is less than 64 bytes
fn admin_unadmin_by_message(
    conn: &mut SqliteConnection,
    message: &Message,
    author: &Users,
) -> String {
    let ctx = languages_ctx();
    if author.is_admin {
        if let Some(user_id) = get_author_id(message) {
            admin_unadmin(user_id, author, conn)
        } else {
            get_text!(ctx, &author.language, "INVALID_ID_ERROR")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "ADMIN_COMMAND_ERROR")
            .unwrap()
            .to_string()
    }
}

/// ban/unban user by telegram id
/// > Note: All `String` is less than 64 bytes
fn ban_unban(user_id: i64, author: &Users, conn: &mut SqliteConnection) -> String {
    let ctx = languages_ctx();
    if user_id.eq(&author.telegram_id.parse::<i64>().unwrap()) {
        get_text!(ctx, &author.language, "CANNOT_BAN_YOURSELF")
            .unwrap()
            .to_string()
    } else if let Some(mut user) = Users::get_by_telegram_id(conn, user_id.to_string()) {
        if user.is_admin && author.telegram_id.ne(&rpg_db::super_user_id().to_string()) {
            get_text!(ctx, &author.language, "CANNOT_BAN_ADMIN")
                .unwrap()
                .to_string()
        } else if user.switch_ban_stutes(conn).is_err() {
            // If there error
            get_text!(ctx, &author.language, "ERROR_WHILE_DO")
                .unwrap()
                .to_string()
        } else if user.is_ban {
            // This means it was banned
            get_text!(ctx, &author.language, "SUCCESSFULLY_BLOCKED")
                .unwrap()
                .to_string()
        } else {
            // This means it wasn't banned
            get_text!(ctx, &author.language, "SUCCESSFULLY_UNBLOCKED")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "USER_NOT_FOUND")
            .unwrap()
            .to_string()
    }
}

/// ban/unban user by username
/// > Note: All `String` is less than 64 bytes
fn ban_unban_by_username(username: &str, author: &Users, conn: &mut SqliteConnection) -> String {
    let ctx = languages_ctx();
    if author.is_admin {
        if let Some(user) = Users::get_by_telegram_username(conn, username) {
            // `0` here will appear user not found message (:
            ban_unban(user.telegram_id.parse().unwrap_or(0), author, conn)
        } else {
            get_text!(ctx, &author.language, "INVALID_ID_ERROR")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "ADMIN_COMMAND_ERROR")
            .unwrap()
            .to_string()
    }
}

/// ban/unban user by message
/// > Note: All `String` is less than 64 bytes
fn ban_unban_by_message(conn: &mut SqliteConnection, message: &Message, author: &Users) -> String {
    let ctx = languages_ctx();
    if author.is_admin {
        if let Some(user_id) = get_author_id(message) {
            ban_unban(user_id, author, conn)
        } else {
            get_text!(ctx, &author.language, "INVALID_ID_ERROR")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "ADMIN_COMMAND_ERROR")
            .unwrap()
            .to_string()
    }
}

/// Return info from message object, use `get_author_id`
fn info_text_by_message(conn: &mut SqliteConnection, message: &Message, author: &Users) -> String {
    let ctx = languages_ctx();
    if let Some(user_id) = get_author_id(message) {
        if user_id.eq(&message.from().unwrap().id) {
            info_text(author, &author.language, conn)
        } else if author.is_admin {
            if let Some(user) = Users::get_by_telegram_id(conn, user_id.to_string()) {
                info_text(&user, &author.language, conn)
            } else {
                get_text!(ctx, &author.language, "USER_NOT_FOUND")
                    .unwrap()
                    .to_string()
            }
        } else {
            get_text!(ctx, &author.language, "REPLY_FOR_ADMIN_ONLY")
                .unwrap()
                .to_string()
        }
    } else {
        get_text!(ctx, &author.language, "INVALID_ID_ERROR")
            .unwrap()
            .to_string()
    }
}

/// Return the ID of sender or owner of the message that was replied to, if any, is returned
/// return `None` if author of forward message is anonymous
fn get_author_id(message: &Message) -> Option<i64> {
    if let Some(reply_message) = message.reply_to_message() {
        if let Some(forward_message) = reply_message.forward() {
            match &forward_message.from {
                ForwardedFrom::User(from) => Some(from.id),
                ForwardedFrom::Chat(from) => Some(from.id),
                ForwardedFrom::SenderName(_) => {
                    // The author of the forwarded message has privacy enabled
                    None
                }
            }
        } else {
            Some(reply_message.from().unwrap().id)
        }
    } else {
        Some(message.from().unwrap().id)
    }
}

/// Return a message containing the status of the ID, the ID of
/// the owner of the message that was replied to, if any, is returned
fn get_author_id_message(message: &Message, language: &str) -> String {
    let ctx = languages_ctx();
    if let Some(user_id) = get_author_id(message) {
        if user_id.eq(&message.from().unwrap().id) {
            format!(
                "{}: `{}`",
                get_text!(ctx, language, "YOUR_ID_MESSAGE").unwrap(),
                user_id
            )
        } else {
            format!(
                "{}: `{}`",
                get_text!(ctx, language, "AUTHOR_ID_MESSAGE").unwrap(),
                user_id
            )
        }
    } else {
        get_text!(ctx, language, "INVALID_ID_ERROR")
            .unwrap()
            .to_string()
    }
}

async fn users_admin_answer(
    bot: &AutoSend<Bot>,
    args: &mut std::vec::IntoIter<&str>,
    author: &Users,
    message_id: i32,
    chat_id: i64,
    query_id: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    try_join!(
        bot.answer_callback_query(query_id)
            .text(admin_unadmin(
                args.next()
                    .ok_or("ban command without telegram id")?
                    .parse::<i64>()?,
                author,
                conn,
            ))
            .send(),
        // FIXME: Dont update if no need to use method as in `update_options` function
            // that fix `MessageNotModified` Error
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboards::admin_users_keyboard(
                conn,
                author.telegram_id.parse::<i64>()?,
                &author.language,
                args.next()
                    .ok_or("ban command without page number")?
                    .parse::<u32>()?,
            )?)
            .send()
    )?;
    Ok(())
}

async fn users_ban_answer(
    bot: &AutoSend<Bot>,
    args: &mut std::vec::IntoIter<&str>,
    author: &Users,
    message_id: i32,
    chat_id: i64,
    query_id: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    try_join!(
        bot.answer_callback_query(query_id)
            .text(ban_unban(
                args.next()
                    .ok_or("ban command without telegram id")?
                    .parse::<i64>()?,
                author,
                conn,
            ))
            .send(),
        // FIXME: Dont update if no need to use method as in `update_options` function
            // that fix `MessageNotModified` Error
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboards::admin_users_keyboard(
                conn,
                author.telegram_id.parse::<i64>()?,
                &author.language,
                args.next()
                    .ok_or("ban command without page number")?
                    .parse::<u32>()?,
            )?)
            .send()
    )?;
    Ok(())
}

/// Run and Share command handler
pub async fn command_handler(
    bot: &AutoSend<Bot>,
    message: &Message,
    command: &Command,
    author: &Users,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Share and Run command need reply message
    if message.reply_to_message().is_some() {
        share_run_answer_message(bot, message, command, author, conn)
            .await
            .log_on_error()
            .await;
    } else {
        let ctx = languages_ctx();
        // If there is no reply message
        bot.send_message(
            message.chat.id,
            get_text!(ctx, &author.language, "REPLY_MESSAGE")
                .unwrap()
                .to_string(),
        )
        .reply_to_message_id(message.id)
        .send()
        .await
        .log_on_error()
        .await;
    };

    Ok(())
}

async fn users_command_handler(
    bot: &AutoSend<Bot>,
    message: &Message,
    author: &Users,
    args: &mut std::vec::IntoIter<&str>,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ctx = languages_ctx();
    if let Some(users_command) = args.next() {
        match users_command {
            "ban" => {
                // The `ban` command is a service, it is not only used to block
                // This means that you can use the `ban` command to block and unblock.
                // how? If the command is used with a banned user, this means canceling his ban and vice versa
                bot.send_message(
                    message.chat.id,
                    if let Some(user_id_or_username) = args.next() {
                        if let Ok(user_id) = user_id_or_username.parse::<i64>() {
                            // ban/unban user by telegram id
                            ban_unban(user_id, author, conn)
                        } else {
                            // ban/unban user by telegram username
                            ban_unban_by_username(user_id_or_username, author, conn)
                        }
                    } else {
                        ban_unban_by_message(conn, message, author)
                    },
                )
                .reply_to_message_id(message.id)
                .send()
                .await
                .log_on_error()
                .await;
            }
            // `admin` command same `ban` is service.
            "admin" => {
                bot.send_message(
                    message.chat.id,
                    if let Some(user_id_or_username) = args.next() {
                        if let Ok(user_id) = user_id_or_username.parse::<i64>() {
                            // admin/unadmin user by telegram id
                            admin_unadmin(user_id, author, conn)
                        } else {
                            // admin/unadmin user by telegram username
                            admin_unadmin_by_username(user_id_or_username, author, conn)
                        }
                    } else {
                        admin_unadmin_by_message(conn, message, author)
                    },
                )
                .reply_to_message_id(message.id)
                .send()
                .await
                .log_on_error()
                .await;
            }
            _ => (),
        };
    } else {
        // Check if the chat is private
        if message.chat.is_private() {
            match keyboards::admin_users_keyboard(
                conn,
                message.from().unwrap().id,
                &author.language,
                0,
            ) {
                Ok(keyboard) => {
                    bot.send_message(
                        message.chat.id,
                        format!(
                            "{} 👮‍♂️",
                            get_text!(ctx, &author.language, "ADMIN_USERS_MESSAGE").unwrap()
                        ),
                    )
                    .reply_to_message_id(message.id)
                    .reply_markup(keyboard)
                    .send()
                    .await?;
                }
                Err(err) => {
                    // This error will appear when the number of users exceeds `18446744073709551615` users. :/
                    bot.send_message(message.chat.id, err.to_string())
                        .reply_to_message_id(message.id)
                        .send()
                        .await?;
                }
            }
        } else {
            bot.send_message(
                message.chat.id,
                format!(
                    "{} 👮‍♂️",
                    get_text!(ctx, &author.language, "PUBLIC_ERROR").unwrap()
                ),
            )
            .reply_to_message_id(message.id)
            .send()
            .await?;
        }
    };

    Ok(())
}

async fn admin_callback_handler(
    bot: AutoSend<Bot>,
    args: &mut std::vec::IntoIter<&str>,
    author: &Users,
    query_id: &str,
    message_id: i32,
    chat_id: i64,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(command) = args.next() {
        if command == "users" {
            if let Some(users_command) = args.next() {
                match users_command {
                    "ban" => {
                        users_ban_answer(&bot, args, author, message_id, chat_id, query_id, conn)
                            .await
                            .log_on_error()
                            .await
                    }
                    "admin" => {
                        users_admin_answer(&bot, args, author, message_id, chat_id, query_id, conn)
                            .await
                            .log_on_error()
                            .await
                    }
                    _ => {}
                }
            }
        }
    };
    Ok(())
}

async fn admin_handler(
    bot: AutoSend<Bot>,
    message: Message,
    author: Users,
    args: Vec<&str>,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ctx = languages_ctx();
    let mut args = args.into_iter();
    if author.is_admin {
        // Handel the admin commands
        if let Some(admin_command) = args.next() {
            if admin_command.eq("users") {
                users_command_handler(&bot, &message, &author, &mut args, conn).await?;
            } else {
                todo!("broadcast, settings");
            };
        } else {
            // Send main admin interface if there no arguments
            // Check if the chat is private
            if message.chat.is_private() {
                bot.send_message(
                    message.chat.id,
                    format!(
                        "{} 👮‍♂️",
                        get_text!(ctx, &author.language, "ADMIN_MAIN_MESSAGE").unwrap()
                    ),
                )
                .reply_to_message_id(message.id)
                .reply_markup(keyboards::admin_main_keybard(&author.language))
                .send()
                .await?;
            } else {
                // Cannot send admin interface in public chat
                bot.send_message(
                    message.chat.id,
                    format!(
                        "{} 👮‍♂️",
                        get_text!(ctx, &author.language, "PUBLIC_ERROR").unwrap()
                    ),
                )
                .reply_to_message_id(message.id)
                .send()
                .await?;
            }
        }
    } else {
        // If the author not admin
        bot.send_message(
            message.chat.id,
            format!(
                "{} ✖️",
                get_text!(ctx, &author.language, "ADMIN_COMMAND_ERROR").unwrap()
            ),
        )
        .reply_to_message_id(message.id)
        .send()
        .await?;
    }
    Ok(())
}

fn get_message_text(
    conn: &mut SqliteConnection,
    args: std::vec::IntoIter<&str>,
    author: &Users,
) -> Option<String> {
    let mut args = args;
    if let Some(command) = args.next() {
        let ctx = languages_ctx();
        match command {
            "admin" => Some(format!(
                "{} 👮‍♂️",
                get_text!(ctx, &author.language, "ADMIN_MAIN_MESSAGE").unwrap()
            )),
            "users" => Some(format!(
                "{} 👮‍♂️",
                get_text!(ctx, &author.language, "ADMIN_USERS_MESSAGE").unwrap()
            )),
            "users-info" => {
                let user: Option<Users> = Users::get_by_telegram_id(conn, args.next()?.into());
                if let Some(user) = user {
                    Some(info_text(&user, &author.language, conn))
                } else {
                    Some(
                        get_text!(ctx, &author.language, "USER_NOT_FOUND")
                            .unwrap()
                            .to_string(),
                    )
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

fn get_keyboard(
    conn: &mut SqliteConnection,
    args: std::vec::IntoIter<&str>,
    author: &Users,
) -> Option<InlineKeyboardMarkup> {
    let mut args = args;
    if let Some(interface) = args.next() {
        match interface {
            "admin" => Some(keyboards::admin_main_keybard(&author.language)),
            "users" => keyboards::admin_users_keyboard(
                conn,
                author
                    .telegram_id
                    .parse::<i64>()
                    .expect("telegram_id should be integer"),
                &author.language,
                args.next()
                    .unwrap_or("0")
                    .parse::<u32>()
                    .expect("page number should be unsigned integer"),
            )
            .ok(),
            "users-info" => Some(keyboards::admin_users_info_keybard(
                args.nth(1).unwrap_or("0"),
                &author.language,
            )),
            _ => None,
        }
    } else {
        None
    }
}

pub async fn message_text_handler(message: Message, bot: AutoSend<Bot>) {
    if let Some(text) = message.text() {
        let conn: &mut SqliteConnection = &mut rpg_db::establish_connection();

        let mut author: Users = rpg_db::get_user(conn, message.from().unwrap()).unwrap();

        if let Some((command, args)) = parse_command(
            &text.to_ascii_lowercase(),
            bot_username(&bot).await.to_ascii_lowercase(),
        ) {
            let command: String = command.to_ascii_lowercase();
            let ctx = languages_ctx();
            if author.can_send_command(conn) {
                author
                    .update(message.from().unwrap(), conn)
                    .await
                    .log_on_error()
                    .await;

                if ["run", "share"].contains(&command.as_ref()) {
                    if message.reply_to_message().is_some() {
                        // for run and share command should have reply message to work.
                        // make record if command are work ( if there reply message )
                        author.make_command_record(conn).log_on_error().await;
                    };
                    let mut code_args = get_args(args).into_iter();
                    if command == "run" {
                        command_handler(
                            &bot,
                            &message,
                            &Command::Run {
                                version: code_args.next().unwrap(),
                                mode: code_args.next().unwrap(),
                                edition: code_args.next().unwrap(),
                            },
                            &author,
                            conn,
                        )
                        .await
                        .log_on_error()
                        .await;
                    } else {
                        command_handler(
                            &bot,
                            &message,
                            &Command::Share {
                                version: code_args.next().unwrap(),
                                mode: code_args.next().unwrap(),
                                edition: code_args.next().unwrap(),
                            },
                            &author,
                            conn,
                        )
                        .await
                        .log_on_error()
                        .await;
                    };

                // for this commands no need to make record
                } else if command == "help" {
                    let mut vars: HashMap<String, String> = HashMap::new();
                    vars.insert(
                        "help_help".to_string(),
                        get_text!(ctx, &author.language, "HELP_HELP")
                            .unwrap()
                            .to_string(),
                    );
                    vars.insert(
                        "help_run".to_string(),
                        get_text!(ctx, &author.language, "RUN_HELP")
                            .unwrap()
                            .to_string(),
                    );
                    vars.insert(
                        "help_share".to_string(),
                        get_text!(ctx, &author.language, "SHARE_HELP")
                            .unwrap()
                            .to_string(),
                    );
                    vars.insert(
                        "help_language".to_string(),
                        get_text!(ctx, &author.language, "LANGUAGE_HELP")
                            .unwrap()
                            .to_string(),
                    );
                    vars.insert(
                        "help_id".to_string(),
                        get_text!(ctx, &author.language, "ID_HELP")
                            .unwrap()
                            .to_string(),
                    );
                    vars.insert(
                        "help_info".to_string(),
                        get_text!(ctx, &author.language, "INFO_HELP")
                            .unwrap()
                            .to_string(),
                    );

                    bot.send_message(
                        message.chat.id,
                        if !args.is_empty() && args[0] == "run" {
                            vars.get("help_run").unwrap().to_string()
                        } else if !args.is_empty() && args[0] == "share" {
                            vars.get("help_share").unwrap().to_string()
                        } else if !args.is_empty() && args[0] == "help" {
                            vars.get("help_help").unwrap().to_string()
                        } else if !args.is_empty() && args[0] == "language" {
                            vars.get("help_language").unwrap().to_string()
                        } else {
                            strfmt(
                                &get_text!(ctx, &author.language, "HELP_MESSAGE")
                                    .unwrap()
                                    .to_string(),
                                &vars,
                            )
                            .unwrap()
                        },
                    )
                    .reply_to_message_id(message.id)
                    .send()
                    .await
                    .log_on_error()
                    .await;
                } else if command == "start" {
                    let mut vars: HashMap<String, String> = HashMap::new();
                    vars.insert("bot_username".to_string(), bot_username(&bot).await);
                    vars.insert(
                        "attempts_maximum".to_string(),
                        (author.attempts_maximum - author.attempts).to_string(),
                    );
                    vars.insert(
                        "command_delay".to_string(),
                        Config::get_or_add("command_delay", "15", conn).value,
                    );
                    vars.insert(
                        "button_delay".to_string(),
                        Config::get_or_add("button_delay", "2", conn).value,
                    );

                    bot.send_message(
                        message.chat.id,
                        strfmt(
                            &get_text!(ctx, &author.language, "START_MESSAGE")
                                .unwrap()
                                .to_string(),
                            &vars,
                        )
                        .unwrap(),
                    )
                    .reply_to_message_id(message.id)
                    .parse_mode(ParseMode::Html)
                    .disable_web_page_preview(true)
                    .reply_markup(keyboards::repo_keyboard(&author.language))
                    .send()
                    .await
                    .log_on_error()
                    .await;
                } else if command == "language" {
                    author.make_command_record(conn).log_on_error().await;
                    bot.send_message(
                        message.chat.id,
                        get_text!(ctx, &author.language, "NEW_LANGUAGE_MESSAGE")
                            .unwrap()
                            .to_string()
                            + " 🤖",
                    )
                    .reply_to_message_id(message.id)
                    .reply_markup(keyboards::languages_keyboard(&author.language))
                    .send()
                    .await
                    .log_on_error()
                    .await;
                } else if command == "info" {
                    author.make_command_record(conn).log_on_error().await;
                    bot.send_message(
                        message.chat.id,
                        info_text_by_message(conn, &message, &author),
                    )
                    .reply_to_message_id(message.id)
                    .send()
                    .await
                    .log_on_error()
                    .await
                } else if command == "id" {
                    author.make_command_record(conn).log_on_error().await;
                    bot.send_message(
                        message.chat.id,
                        get_author_id_message(&message, &author.language),
                    )
                    .reply_to_message_id(message.id)
                    .parse_mode(ParseMode::MarkdownV2)
                    .send()
                    .await
                    .log_on_error()
                    .await
                } else if command == "admin" {
                    author.make_command_record(conn).log_on_error().await;
                    admin_handler(bot, message, author, args, conn)
                        .await
                        .log_on_error()
                        .await;
                }
            } else {
                // Cannot send command
                bot.send_message(
                    message.chat.id,
                    if author.is_ban {
                        get_text!(ctx, &author.language, "BAN_MESSAGE")
                            .unwrap()
                            .to_string()
                    } else if author.attempts >= author.attempts_maximum {
                        attempt_error_message(&author)
                    } else {
                        delay_error_message(&author, true, conn)
                    },
                )
                .reply_to_message_id(message.id)
                .send()
                .await
                .log_on_error()
                .await;
            };
        } else {
            // Not command (Text)
        };
    }
}

pub async fn callback_handler(bot: AutoSend<Bot>, callback_query: CallbackQuery) {
    // callback data be like this in all callback
    //
    // <command> <args> <args> ..
    // viewR <code> <already_use_keyboard>
    // viewS <code> <already_use_keyboard>
    // print <message_with_underscore>
    // run <code>
    // share <code>
    // option <code> <option_name> <option_value>
    // change_lang <new_language>
    // gotok <interface> <args: optional>...
    // goto <interface> <args: optional>...
    // admin <command> <args: optional>...

    if let Some(callback_data) = callback_query.data.clone() {
        log::debug!("{callback_data}");
        let conn: &mut SqliteConnection = &mut rpg_db::establish_connection();
        let mut author: Users = rpg_db::get_user(conn, &callback_query.from).unwrap();

        let ctx = languages_ctx();
        if author.can_click_button(conn) {
            // Can click button
            author.make_button_record(conn).log_on_error().await;

            let message: &Message = callback_query.message.as_ref().unwrap();
            let mut args = callback_data
                .split_whitespace()
                .collect::<Vec<&str>>()
                .into_iter();
            let command: &str = args.next().expect("callback_data don't have command");

            match command {
                "viewR" | "viewS" => {
                    view_handler(
                        &bot,
                        &callback_query,
                        command,
                        args.next().expect("viewR/viewS don't have code"),
                        args.next()
                            .expect("viewR/viewS don't have already_use_keyboard")
                            .parse()
                            .unwrap(),
                        &author.language,
                        conn,
                    )
                    .await;
                }

                "print" => {
                    bot.answer_callback_query(&callback_query.id)
                        .text(
                            args.next()
                                .expect("print command don't have message to print it")
                                .replace('_', " "),
                        )
                        .send()
                        .await
                        .log_on_error()
                        .await;
                }

                "run" | "share" => {
                    run_share_callback(
                        &bot,
                        &callback_query,
                        command,
                        args.next()
                            .expect("share/run command don't have code")
                            .to_string(),
                        &author.language,
                        conn,
                    )
                    .await;
                }

                "option" => {
                    update_options(
                        &bot,
                        &callback_query,
                        args.next().expect("option command don't have code"),
                        args.next().expect("option command don't have option_name"),
                        args.next().expect("option command don't have option_value"),
                        &author.language,
                        conn,
                    )
                    .await;
                }
                "change_lang" => {
                    change_language(
                        &bot,
                        &mut author,
                        message.id,
                        message.chat.id,
                        args.next().expect("change_lang don't have new_language"),
                        &callback_query.id,
                        conn,
                    )
                    .await;
                }

                "gotok" => {
                    let message: &Message = callback_query.message.as_ref().unwrap();
                    bot.edit_message_reply_markup(message.chat.id, message.id)
                        .reply_markup(get_keyboard(conn, args.clone(), &author).unwrap_or_else(
                            || panic!("back_keyboard return `None`, args: {:?}", args),
                        ))
                        .send()
                        .await
                        .log_on_error()
                        .await;
                }
                // TODO: DRY with gotok and goto
                "goto" => {
                    let message: &Message = callback_query.message.as_ref().unwrap();
                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        get_message_text(conn, args.clone(), &author).unwrap_or_else(|| {
                            panic!("get_message_text return `None`, args: {:?}", args)
                        }),
                    )
                    .reply_markup(
                        get_keyboard(conn, args.clone(), &author).unwrap_or_else(|| {
                            panic!("back_keyboard return `None`, args: {:?}", args)
                        }),
                    )
                    .send()
                    .await
                    .log_on_error()
                    .await;
                }

                "admin" => {
                    admin_callback_handler(
                        bot,
                        &mut args,
                        &author,
                        &callback_query.id,
                        message.id,
                        message.chat.id,
                        conn,
                    )
                    .await
                    .log_on_error()
                    .await;
                }
                _ => (),
            };
        } else {
            bot.answer_callback_query(callback_query.id)
                .text(if author.is_ban {
                    get_text!(ctx, &author.language, "BAN_MESSAGE")
                        .unwrap()
                        .to_string()
                } else if author.attempts >= author.attempts_maximum {
                    attempt_error_message(&author)
                } else {
                    delay_error_message(&author, false, conn)
                })
                .send()
                .await
                .log_on_error()
                .await;
        };
    }
}

async fn view_handler(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    view: &str,
    code: &str,
    already_use_keyboard: bool,
    language: &str,
    conn: &mut SqliteConnection,
) {
    if already_use_keyboard {
        already_use_answer(bot, &callback_query.id, language, view == "viewR").await;
    } else if let Ok(source) = SourceCode::get_by_code(code, conn) {
        // unwrap here because every callback query have message 🙂
        let message: Message = callback_query.clone().message.unwrap();
        let keyboard: InlineKeyboardMarkup = if view == "viewR" {
            keyboards::run_keyboard(source, language)
        } else {
            keyboards::share_keyboard(source, language)
        };

        bot.edit_message_reply_markup(message.chat.id, message.id)
            .reply_markup(keyboard)
            .send()
            .await
            .log_on_error()
            .await;
    } else {
        cannot_reached_answer(bot, &callback_query.id, language).await;
    }
}
