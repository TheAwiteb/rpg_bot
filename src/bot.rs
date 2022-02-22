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

use crate::{keyboards, messages, rpg};
use futures::try_join;
use std::error::Error;
use teloxide::payloads::SendMessageSetters;
use teloxide::utils::command::parse_command;
use teloxide::{
    prelude2::*,
    requests::Requester,
    types::{InlineKeyboardMarkup, ParseMode},
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

#[allow(unused_variables)]
async fn share_run_answer(
    bot: &AutoSend<Bot>,
    command: &Command,
    already_use_keyboard: bool,
    message: Message,
    code: rpg::Code,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (keyboard, output): (InlineKeyboardMarkup, String) = if command.name() == "run" {
        let output: Result<String, String> = rpg::run(&code).await;
        (
            keyboards::view_share_keyboard(
                &code.version,
                &code.mode,
                &code.edition,
                already_use_keyboard,
                output.is_ok(),
            ),
            match output {
                Ok(output) => output,
                Err(output) => output,
            },
        )
    } else {
        let output: Result<String, String> = rpg::share(&code).await;
        (
            keyboards::view_run_keyboard(
                &code.version,
                &code.mode,
                &code.edition,
                already_use_keyboard,
                output.is_ok(),
            ),
            match output {
                Ok(output) => output,
                Err(output) => output,
            },
        )
    };
    bot.edit_message_text(
        // For text messages, the actual UTF-8 text of the message, 0-4096 characters
        // https://core.telegram.org/bots/api#message
        message.chat_id(),
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
#[allow(unused_variables)]
pub async fn share_run_answer_message(
    bot: &AutoSend<Bot>,
    message: &Message,
    command: &Command,
    version: &str,
    mode: &str,
    edition: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let code: rpg::Code = rpg::Code::new(
        message.reply_to_message().unwrap().text().unwrap(),
        version,
        mode,
        edition,
    );
    if let Err(err) = code.is_valid() {
        bot.send_message(message.chat.id, err)
            .reply_to_message_id(message.id)
            .send()
            .await?;
    } else {
        let message: Message =
            replay_wait_message(bot, message.chat.id, message.id, command).await?;
        share_run_answer(bot, command, false, message, code).await?;
    }
    Ok(())
}

pub async fn share_run_answer_cllback(
    bot: &AutoSend<Bot>,
    chat_id: i64,
    command: &str,
    source_code: &str,
    version: &str,
    mode: &str,
    edition: &str,
) -> Result<(), RequestError> {
    let code: rpg::Code = rpg::Code::new(source_code, version, mode, edition);
    let message: Message = send_wait_message(bot, chat_id, &Command::from((&code, command)))
        .await
        .unwrap();
    share_run_answer(bot, &Command::from((&code, command)), true, message, code)
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
fn get_source_code() -> Option<&'static str> {
    // TODO: return source code of message
    None
}

async fn run_share_callback(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    command: &str,
    version: &str,
    mode: &str,
    edition: &str,
) {
    // share and run commands need source code
    // share and run commands args is <already_use_keyboard>
    // if get_source_code returns None that mean the source code message is deleted
    if let Some(source_code) = get_source_code() {
        let message: Message = callback_query.clone().message.unwrap();
        let keyboard: InlineKeyboardMarkup = if command == "share" {
            keyboards::view_share_keyboard(version, mode, edition, true, true)
        } else {
            keyboards::view_run_keyboard(version, mode, edition, true, true)
        };
        try_join!(
            share_run_answer_cllback(
                bot,
                message.chat.id,
                command,
                source_code,
                version,
                mode,
                edition
            ),
            bot.edit_message_reply_markup(message.chat_id(), message.id)
                .reply_markup(keyboard)
                .send()
        )
        .unwrap();
    } else {
        bot.answer_callback_query(&callback_query.id)
            .text(messages::MESSAGE_CANNOT_REACHED)
            .send()
            .await
            .unwrap();
    }
}

async fn update_options(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    version: &str,
    mode: &str,
    edition: &str,
    option_name: &str,
    option_value: &str,
) {
    let message: Message = callback_query.clone().message.unwrap();
    let old_keybord: &InlineKeyboardMarkup = message.reply_markup().unwrap();
    let answer = bot
        .answer_callback_query(&callback_query.id)
        .text(format!("set {} to {}", option_name, option_value))
        .send();

    let keyboard: InlineKeyboardMarkup = if old_keybord.inline_keyboard[4][0].text.contains("Run") {
        keyboards::run_keyboard(version, mode, edition)
    } else {
        keyboards::share_keyboard(version, mode, edition)
    };

    if &keyboard != old_keybord {
        try_join!(
            answer,
            bot.edit_message_reply_markup(message.chat_id(), message.id)
                .reply_markup(keyboard)
                .send()
        )
        .log_on_error()
        .await;
    } else {
        answer.await.log_on_error().await;
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
                .await?;
        };
    };

    Ok(())
}

pub async fn message_text_handler(
    message: Message,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = message.text() {
        if let Some((command, args)) = parse_command(text, bot_username(&bot).await) {
            // Is command
            let command: String = command.to_ascii_lowercase();
            if ["run".into(), "share".into()].contains(&command) {
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
                    .await?;
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
                    .await?;
                };
            } else if command == "help" {
                if args.len() > 0 && args[0] == "run" {
                    bot.send_message(message.chat.id, messages::RUN_HELP)
                        .reply_to_message_id(message.id)
                        .send()
                        .await?;
                } else if args.len() > 0 && args[0] == "share" {
                    bot.send_message(message.chat.id, messages::SHARE_HELP)
                        .reply_to_message_id(message.id)
                        .send()
                        .await?;
                } else {
                    bot.send_message(message.chat.id, Command::descriptions())
                        .reply_to_message_id(message.id)
                        .send()
                        .await?;
                }
            } else if command == "start" {
                bot.send_message(message.chat.id, messages::START_MESSAGE)
                    .reply_to_message_id(message.id)
                    .parse_mode(ParseMode::MarkdownV2)
                    .disable_web_page_preview(true)
                    .reply_markup(keyboards::repo_keyboard())
                    .send()
                    .await?;
            };
        } else {
            // Not command (Text)
        };
    }
    Ok(())
}

pub async fn callback_handler(
    bot: AutoSend<Bot>,
    callback_query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // callback data be like this in all callback
    //
    // <command> <args> <args> ..
    // viewR <version> <mode> <edition> <already_use_keyboard>
    // viewS <version> <mode> <edition> <already_use_keyboard>
    // already_use
    // print <message_with_underscore>
    // run <version> <mode> <edition>
    // share <version> <mode> <edition>
    // option <version> <mode> <edition> <option_name> <option_value>

    if let Some(callback_data) = callback_query.data.clone() {
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
                    args.next().expect("viewR/viewS don't have first arg"),
                    args.next().expect("viewR/viewS don't have second arg"),
                    args.next().expect("viewR/viewS don't have third arg"),
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
                    .unwrap();
            }
            "run" | "share" => {
                run_share_callback(
                    &bot,
                    &callback_query,
                    command,
                    args.next().expect("share/run command don't have version"),
                    args.next().expect("share/run command don't have mode"),
                    args.next().expect("share/run command don't have edition"),
                )
                .await;
            }

            "option" => {
                update_options(
                    &bot,
                    &callback_query,
                    args.next().expect("option command don't have version"),
                    args.next().expect("option command don't have mode"),
                    args.next().expect("option command don't have edition"),
                    args.next().expect("option command don't have option_name"),
                    args.next().expect("option command don't have option_value"),
                )
                .await
            }
            _ => (),
        };
    }
    Ok(())
}

async fn view_handler(
    bot: &AutoSend<Bot>,
    callback_query: &CallbackQuery,
    view: &str,
    version: &str,
    mode: &str,
    edition: &str,
    already_use_keyboard: bool,
) {
    if already_use_keyboard {
        already_use_answer(bot, &callback_query.id).await;
    } else {
        // unwrap here because every callback query have message 🙂
        let message: Message = callback_query.clone().message.unwrap();
        let keyboard: InlineKeyboardMarkup = if view == "viewR" {
            keyboards::run_keyboard(version, mode, edition)
        } else {
            keyboards::share_keyboard(version, mode, edition)
        };

        bot.edit_message_reply_markup(message.chat_id(), message.id)
            .reply_markup(keyboard)
            .send()
            .await
            .unwrap();
    }
}
