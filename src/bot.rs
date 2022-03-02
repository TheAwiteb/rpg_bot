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
use crate::{keyboards, messages, models::SourceCode, rpg, rpg_db};
use chrono::offset;
use futures::try_join;
use std::error::Error;
use teloxide::payloads::SendMessageSetters;
use teloxide::utils::command::parse_command;
use teloxide::{
    prelude2::*,
    requests::Requester,
    types::{InlineKeyboardMarkup, ParseMode, User},
    utils::command::BotCommand,
    RequestError,
};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = r#"Display this text, and commands help
        /help <command (default: all)>
        Example:
            /hlep run
            "#)]
    Help,
    #[command(
        description = r#"Reply to message with this command to execute Rust code 🦀⚙️
        /run <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>
        Example:
            /run stable debug 2021
            "#,
        parse_with = "split"
    )]
    Run {
        version: String,
        mode: String,
        edition: String,
    },
    #[command(
        description = r#"Reply to message with this command to share Rust code 🦀🔗
        /share <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>
        Example:
            /share stable debug 2021"#,
        parse_with = "split"
    )]
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
        if command_name.to_ascii_lowercase() == String::from("run") {
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
fn get_wait_message(command: &Command) -> Result<String, String> {
    if let Some((version, mode, edition)) = command.args() {
        let ditals: String = format!("Version: {version}\nMode: {mode}\nEdition: {edition}");
        if command.name() == "run" {
            return Ok(format!("The code is being executed 🦀⚙️\n{ditals}"));
        } else {
            return Ok(format!("Creating a playground URL 🦀🔗\n{ditals}"));
        }
    } else {
        return Err(format!(
            "'{}' is invalid command for `get_wait_message` function",
            command.name()
        ));
    }
}

async fn already_use_answer(requester: &AutoSend<Bot>, query_id: &str) {
    requester
        .answer_callback_query(query_id)
        .text(messages::ALREADY_USE_KEYBOARD)
        .send()
        .await
        .unwrap();
}

async fn replay_wait_message(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    message_id: i32,
    command: &Command,
) -> Result<Message, String> {
    match get_wait_message(command) {
        Ok(message) => Ok(bot
            .send_message(chat_id, message)
            .reply_to_message_id(message_id)
            .send()
            .await
            .map_err(|err| format!("{}", err))?),
        Err(err) => {
            log::error!("{}", err);
            Err(err)
        }
    }
}

async fn send_wait_message(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    command: &Command,
) -> Result<Message, String> {
    match get_wait_message(command) {
        Ok(message) => Ok(bot
            .send_message(chat_id, message)
            .send()
            .await
            .map_err(|err| format!("{}", err))?),
        Err(err) => {
            log::error!("{}", err);
            Err(err)
        }
    }
}

fn delay_error_message(author: &Users, is_command: bool) -> String {
    format!(
        "Sorry, you have to wait {}s (in anticipation of spam)",
        (if is_command {
                author.last_command_record
            } else {
                author.last_button_record
            }
            .unwrap() // The use of unwrap here is normal, because if no record is made to the user, the
                        // `can_send_command` and `can_click_button` functions will return `true`.
            .timestamp()
            // TODO: Use db to get delay
            + if is_command { 15 } else { 2 })
            - (offset::Utc::now().timestamp())
    )
}

fn attempt_error_message() -> String {
    format!(
        "Sorry, you have exceeded {} bot attempts ❗",
        100 // TODO: get it from db
    )
}

/// Share and run, and make attempt for user
async fn share_run_answer(
    bot: &AutoSend<Bot>,
    command: &Command,
    already_use_keyboard: bool,
    message: &Message,
    author: &mut Users,
    code: rpg::Code,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let output: Result<String, String> = if command.name() == "run" {
        rpg::run(&code).await
    } else {
        rpg::share(&code).await
    };

    let code: Option<String> = if output.is_ok() {
        Some(
            rpg_db::create_source(&mut rpg_db::establish_connection(), &code, author)
                .await
                .unwrap()
                .code,
        )
    } else {
        None
    };
    let (keyboard, output): (InlineKeyboardMarkup, String) = if command.name() == "run" {
        (
            keyboards::view_share_keyboard(code, already_use_keyboard, output.is_ok()),
            match output {
                Ok(output) => output,
                Err(output) => output,
            },
        )
    } else {
        (
            keyboards::view_run_keyboard(code, already_use_keyboard, output.is_ok()),
            match output {
                Ok(output) => output,
                Err(output) => output,
            },
        )
    };
    author
        .make_attempt(&mut rpg_db::establish_connection())
        .await
        .unwrap();
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
    .await?;

    Ok(())
}

/// Send code output for run command and Rust playground for share command
pub async fn share_run_answer_message(
    bot: &AutoSend<Bot>,
    message: &Message,
    command: &Command,
    version: &str,
    mode: &str,
    edition: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    SourceCode::filter_source_codes(&mut rpg_db::establish_connection()).unwrap();
    let source_code_message: &Message = message.reply_to_message().unwrap();
    if let Some(source_code) = source_code_message.text() {
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
                replay_wait_message(bot, message.chat.id, message.id, command).await?;
            share_run_answer(
                bot,
                command,
                false,
                &reply_message,
                &mut rpg_db::get_user(
                    &mut rpg_db::establish_connection(),
                    &message.from().unwrap(),
                )
                .await
                .unwrap(),
                code,
            )
            .await
            .log_on_error()
            .await;
        }
    } else {
        bot.send_message(message.chat.id, messages::MUST_BE_TEXT)
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
) -> Result<(), RequestError> {
    let message: Message = send_wait_message(bot, chat_id, &Command::from((&code, command)))
        .await
        .unwrap();
    share_run_answer(
        bot,
        &Command::from((&code, command)),
        true,
        &message,
        &mut rpg_db::get_user(&mut rpg_db::establish_connection(), author)
            .await
            .unwrap(),
        code,
    )
    .await
    .unwrap();
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
fn get_source_code(code: &str) -> Option<SourceCode> {
    SourceCode::get_by_code(code, &mut rpg_db::establish_connection()).ok()
}

async fn cannot_reached_answer(bot: &AutoSend<Bot>, query_id: &str) {
    bot.answer_callback_query(query_id)
        .text(messages::MESSAGE_CANNOT_REACHED)
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
) {
    // share and run commands need source code
    // if get_source_code returns None that mean the source code message is deleted
    if let Some(source_code) = get_source_code(&code) {
        let message: Message = callback_query.clone().message.unwrap();
        let keyboard: InlineKeyboardMarkup = if command == "share" {
            keyboards::view_share_keyboard(Some(code), true, true)
        } else {
            keyboards::view_run_keyboard(Some(code), true, true)
        };
        try_join!(
            share_run_answer_cllback(
                bot,
                message.chat.id,
                command,
                source_code.into(),
                &callback_query.from
            ),
            bot.edit_message_reply_markup(message.chat.id, message.id)
                .reply_markup(keyboard)
                .send()
        )
        .log_on_error()
        .await;
    } else {
        cannot_reached_answer(&bot, &callback_query.id).await;
    }
}

async fn update_options(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    code: &str,
    option_name: &str,
    option_value: &str,
) {
    if let Ok(mut source) = SourceCode::get_by_code(code, &mut rpg_db::establish_connection()) {
        let message: Message = callback_query.clone().message.unwrap();
        let old_keybord: &InlineKeyboardMarkup = message.reply_markup().unwrap();
        let answer = bot
            .answer_callback_query(&callback_query.id)
            .text(format!("set {} to {}", option_name, option_value))
            .send();

        source
            .update_by_name(
                option_name,
                option_value,
                &mut rpg_db::establish_connection(),
            )
            .log_on_error()
            .await;

        let keyboard: InlineKeyboardMarkup =
            if old_keybord.inline_keyboard[4][0].text.contains("Run") {
                keyboards::run_keyboard(source)
            } else {
                keyboards::share_keyboard(source)
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
        cannot_reached_answer(bot, &callback_query.id).await;
    }
}

/// Run and Share command handler
pub async fn command_handler(
    bot: &AutoSend<Bot>,
    message: &Message,
    command: &Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some((version, mode, edition)) = command.args() {
        // Share and Run command need reply message
        if message.reply_to_message().is_some() {
            share_run_answer_message(bot, message, command, version, mode, edition).await?;
        } else {
            // If there is no reply message
            bot.send_message(message.chat.id, messages::REPLY_MESSAGE)
                .reply_to_message_id(message.id)
                .send()
                .await
                .log_on_error()
                .await;
        };
    };

    Ok(())
}

pub async fn message_text_handler(message: Message, bot: AutoSend<Bot>) {
    if let Some(text) = message.text() {
        let mut author: Users = rpg_db::get_user(
            &mut rpg_db::establish_connection(),
            &message.from().unwrap(),
        )
        .await
        .unwrap();

        if let Some((command, args)) = parse_command(text, bot_username(&bot).await) {
            // Is command
            if author.can_send_command() || message.reply_to_message().is_none() {
                // we have two command need to make record of them, (`run`, `share`), `run` and `share` commands need reply message to work
                // Can send command

                let command: String = command.to_ascii_lowercase();
                if ["run".into(), "share".into()].contains(&command) {
                    if message.reply_to_message().is_some() {
                        // for run and share command should have reply message to work.
                        // make record if command are work ( if there reply message )
                        author
                            .make_command_record(&mut rpg_db::establish_connection())
                            .await
                            .log_on_error()
                            .await;
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
                        )
                        .await
                        .log_on_error()
                        .await;
                    };

                // for this commands no need to make record
                } else if command == "help" {
                    if args.len() > 0 && args[0] == "run" {
                        bot.send_message(message.chat.id, messages::RUN_HELP)
                            .reply_to_message_id(message.id)
                            .send()
                            .await
                            .log_on_error()
                            .await;
                    } else if args.len() > 0 && args[0] == "share" {
                        bot.send_message(message.chat.id, messages::SHARE_HELP)
                            .reply_to_message_id(message.id)
                            .send()
                            .await
                            .log_on_error()
                            .await;
                    } else {
                        bot.send_message(message.chat.id, Command::descriptions())
                            .reply_to_message_id(message.id)
                            .send()
                            .await
                            .log_on_error()
                            .await;
                    }
                } else if command == "start" {
                    bot.send_message(message.chat.id, format!(
                        "{}\nNote:\nYou have {} attempts to use bot \\(share and run\\)\n{} seconds between every command\n{} seconds between every button click",
                    // TODO: get delay and attempts from db
                    messages::START_MESSAGE, 100, 15, 2
                )
                    )
                        .reply_to_message_id(message.id)
                        .parse_mode(ParseMode::MarkdownV2)
                        .disable_web_page_preview(true)
                        .reply_markup(keyboards::repo_keyboard())
                        .send()
                        .await
                        .log_on_error()
                        .await;
                };
            } else {
                // Cannot send command
                // TODO: Use db to get attempts
                bot.send_message(
                    message.chat.id,
                    if author.attempts == 100 {
                        attempt_error_message()
                    } else {
                        delay_error_message(&author, true)
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
    // already_use
    // print <message_with_underscore>
    // run <code>
    // share <code>
    // option <code> <option_name> <option_value>

    if let Some(callback_data) = callback_query.data.clone() {
        let mut author: Users =
            rpg_db::get_user(&mut rpg_db::establish_connection(), &callback_query.from)
                .await
                .unwrap();

        if author.can_click_button() {
            // Can click button
            author
                .make_button_record(&mut rpg_db::establish_connection())
                .await
                .log_on_error()
                .await;

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
                            .expect("viewR/viewS don't have fourth arg")
                            .parse()
                            .unwrap(),
                    )
                    .await
                }

                "already_use" | "print" => {
                    bot.answer_callback_query(&callback_query.id)
                        .text(if command == "print" {
                            args.next()
                                .expect("print command don't have message to print it")
                                .replace('_', " ")
                        } else {
                            messages::ALREADY_USE_KEYBOARD.to_string()
                        })
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
                    )
                    .await
                }
                _ => (),
            };
        } else {
            bot.answer_callback_query(callback_query.id)
                .text(if author.attempts == 100 {
                    attempt_error_message()
                } else {
                    delay_error_message(&author, false)
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
) {
    if already_use_keyboard {
        already_use_answer(bot, &callback_query.id).await;
    } else {
        if let Ok(source) = SourceCode::get_by_code(code, &mut rpg_db::establish_connection()) {
            // unwrap here because every callback query have message 🙂
            let message: Message = callback_query.clone().message.unwrap();
            let keyboard: InlineKeyboardMarkup = if view == "viewR" {
                keyboards::run_keyboard(source)
            } else {
                keyboards::share_keyboard(source)
            };

            bot.edit_message_reply_markup(message.chat.id, message.id)
                .reply_markup(keyboard)
                .send()
                .await
                .unwrap();
        } else {
            cannot_reached_answer(bot, &callback_query.id).await;
        }
    }
}
