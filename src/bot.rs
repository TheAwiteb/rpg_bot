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
    models::{Config, SourceCode},
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
    requests::Requester,
    types::{InlineKeyboardMarkup, ParseMode, User},
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

impl From<(&rpg::Code, &str)> for Command {
    fn from((code, command_name): (&rpg::Code, &str)) -> Command {
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
    code: rpg::Code,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let output: Result<String, String> = if command.name() == "run" {
        rpg::run(&code).await
    } else {
        rpg::share(&code).await
    };

    let code: Option<String> = if output.is_ok() {
        Some(rpg_db::create_source(conn, &code, author).unwrap().code)
    } else {
        None
    };
    let (keyboard, output): (InlineKeyboardMarkup, String) = if command.name() == "run" {
        (
            keyboards::view_share_keyboard(
                code,
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
                code,
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
    language: &str,
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
                let reply_message: Message =
                    replay_wait_message(bot, message.chat.id, message.id, command, language)
                        .await?;
                share_run_answer(
                    bot,
                    command,
                    false,
                    &reply_message,
                    &mut rpg_db::get_user(conn, message.from().unwrap()).unwrap(),
                    code,
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
            get_text!(ctx, language, "MUST_BE_TEXT")
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

pub async fn share_run_answer_cllback(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    command: &str,
    code: rpg::Code,
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
        code,
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
            keyboards::view_share_keyboard(Some(code), true, true, language)
        } else {
            keyboards::view_run_keyboard(Some(code), true, true, language)
        };
        try_join!(
            share_run_answer_cllback(
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
        let old_keybord: &InlineKeyboardMarkup = message.reply_markup().unwrap();
        let answer = bot
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
            if old_keybord.inline_keyboard[4][0].text.contains("Run") {
                keyboards::run_keyboard(source, language)
            } else {
                keyboards::share_keyboard(source, language)
            };

        if &keyboard != old_keybord {
            try_join!(
                answer,
                bot.edit_message_reply_markup(message.chat.id, message.id)
                    .reply_markup(keyboard)
                    .send()
            )
            .log_on_error()
            .await;
        } else {
            answer.await.log_on_error().await;
        }
    } else {
        cannot_reached_answer(bot, &callback_query.id, language).await;
    }
}

/// Answer to change langauge request (CallbackQuery and Message)
async fn change_langauge(
    bot: &AutoSend<Bot>,
    author: &mut Users,
    message_id: i32,
    chat_id: i64,
    new_language: &str,
    query_id: &str,
    conn: &mut SqliteConnection,
) {
    let ctx = languages_ctx();
    let new_language = new_language.replace("_", " ");

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

pub fn info_text(author: &Users, conn: &mut SqliteConnection) -> String {
    let ctx = languages_ctx();
    let mut vars: HashMap<String, String> = HashMap::new();
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
        &get_text!(ctx, &author.language, "INFO_MESSAGE")
            .unwrap()
            .to_string(),
        &vars,
    )
    .unwrap()
}

/// Run and Share command handler
pub async fn command_handler(
    bot: &AutoSend<Bot>,
    message: &Message,
    command: &Command,
    language: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Share and Run command need reply message
    if message.reply_to_message().is_some() {
        share_run_answer_message(bot, message, command, language, conn)
            .await
            .log_on_error()
            .await;
    } else {
        let ctx = languages_ctx();
        // If there is no reply message
        bot.send_message(
            message.chat.id,
            get_text!(ctx, language, "REPLY_MESSAGE")
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

pub async fn message_text_handler(message: Message, bot: AutoSend<Bot>) {
    if let Some(text) = message.text() {
        let conn: &mut SqliteConnection = &mut rpg_db::establish_connection();

        let mut author: Users = rpg_db::get_user(conn, message.from().unwrap()).unwrap();

        if let Some((command, args)) = parse_command(
            &text.to_ascii_lowercase(),
            bot_username(&bot).await.to_ascii_lowercase(),
        ) {
            let command: String = command.to_ascii_lowercase();
            if author.can_send_command(conn)
                || (["run", "share"].contains(&command.as_ref())
                    && message.reply_to_message().is_none())
            {
                let ctx = languages_ctx();
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
                            &author.language,
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
                            &author.language,
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
                    bot.send_message(message.chat.id, info_text(&author, conn))
                        .reply_to_message_id(message.id)
                        .send()
                        .await
                        .log_on_error()
                        .await
                };
            } else {
                // Cannot send command
                bot.send_message(
                    message.chat.id,
                    if author.attempts >= author.attempts_maximum {
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

    if let Some(callback_data) = callback_query.data.clone() {
        let conn: &mut SqliteConnection = &mut rpg_db::establish_connection();
        let mut author: Users = rpg_db::get_user(conn, &callback_query.from).unwrap();

        if author.can_click_button(conn) {
            // Can click button
            author.make_button_record(conn).log_on_error().await;

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
                    let message: Message = callback_query.message.unwrap();
                    change_langauge(
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
                _ => (),
            };
        } else {
            bot.answer_callback_query(callback_query.id)
                .text(if author.attempts >= author.attempts_maximum {
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
            .unwrap();
    } else {
        cannot_reached_answer(bot, &callback_query.id, language).await;
    }
}
