// rpg_bot - Telegram bot help you to run Rust code in Telegram via Rust playground
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
    prelude::*,
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

fn get_wait_message(command: &Command) -> String {
    match &command {
        Command::Run {
            version,
            mode,
            edition,
        } => {
            format!(
                r#"The code is being executed 🦀⚙️
        Version: {version}
        Mode: {mode}
        Edition: {edition}"#
            )
        }
        Command::Share {
            version,
            mode,
            edition,
        } => {
            format!(
                r#"Creating a playground URL 🦀🔗
        Version: {version}
        Mode: {mode}
        Edition: {edition}"#
            )
        }
        _ => "".into(),
    }
}

async fn already_use_answer(requester: &AutoSend<Bot>, query_id: String) {
    requester
        .answer_callback_query(query_id)
        .text(messages::ALREADY_USE_KEYBOARD)
        .send()
        .await
        .unwrap();
}

async fn replay_wait_message(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    command: &Command,
) -> Result<Message, Box<dyn Error + Send + Sync>> {
    let message: String = get_wait_message(command);
    Ok(cx.reply_to(message).send().await?)
}

async fn send_wait_message(
    cx: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    command: &Command,
) -> Result<Message, Box<dyn Error + Send + Sync>> {
    let message: String = get_wait_message(command);
    Ok(cx
        .requester
        .send_message(cx.update.clone().message.unwrap().chat_id(), message)
        .send()
        .await?)
}

#[allow(unused_variables)]
async fn share_run_answer(
    requester: &AutoSend<Bot>,
    command: &Command,
    already_use_keyboard: bool,
    message: Message,
    code: rpg::Code,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (output, keyboard): (Option<String>, Option<InlineKeyboardMarkup>) = match command {
        Command::Run {
            version,
            mode,
            edition,
        } => (
            Some(rpg::run(&code).await?),
            Some(keyboards::view_share_keyboard(
                version,
                mode,
                edition,
                already_use_keyboard,
            )),
        ),
        Command::Share {
            version,
            mode,
            edition,
        } => (
            Some(rpg::share(&code).await?),
            Some(keyboards::view_run_keyboard(
                version,
                mode,
                edition,
                already_use_keyboard,
            )),
        ),
        _ => (None, None),
    };
    let (output, keyboard): (String, InlineKeyboardMarkup) = (output.unwrap(), keyboard.unwrap());
    requester
        .edit_message_text(
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
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    command: &Command,
    version: String,
    mode: String,
    edition: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let code: rpg::Code = rpg::Code::new(
        cx.update
            .reply_to_message()
            .unwrap()
            .text()
            .unwrap()
            .to_owned(),
        version,
        mode,
        edition,
    );
    if let Err(err) = code.is_valid() {
        cx.reply_to(err).send().await?;
    } else {
        let message: Message = replay_wait_message(&cx, &command).await?;
        share_run_answer(&cx.requester, command, false, message, code).await?;
    }
    Ok(())
}

pub async fn share_run_answer_cllback(
    cx: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    command: &str,
    source_code: String,
    version: String,
    mode: String,
    edition: String,
) -> Result<(), RequestError> {
    let code: rpg::Code = rpg::Code::new(source_code, version, mode, edition);
    let message: Message = send_wait_message(&cx, &Command::from((&code, command)))
        .await
        .unwrap();
    share_run_answer(
        &cx.requester,
        &Command::from((&code, command)),
        true,
        message,
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
fn get_source_code() -> Option<String> {
    // TODO: return source code of message
    None
}

async fn run_share_callback(
    cx: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    command: &str,
    version: String,
    mode: String,
    edition: String,
) {
    // share and run commands need source code
    // share and run commands args is <already_use_keyboard>
    // if get_source_code returns None that mean the source code message is deleted
    if let Some(source_code) = get_source_code() {
        let message: Message = cx.update.clone().message.unwrap();
        let keyboard: InlineKeyboardMarkup = if command == "share" {
            keyboards::view_share_keyboard(&version, &mode, &edition, true)
        } else {
            keyboards::view_run_keyboard(&version, &mode, &edition, true)
        };
        try_join!(
            share_run_answer_cllback(cx, command, source_code, version, mode, edition),
            cx.requester
                .edit_message_reply_markup(message.chat_id(), message.id)
                .reply_markup(keyboard)
                .send()
        )
        .unwrap();
    } else {
        cx.requester
            .answer_callback_query(cx.update.id.clone())
            .text(messages::MESSAGE_CANNOT_REACHED)
            .send()
            .await
            .unwrap();
    }
}

async fn update_options(
    cx: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    version: String,
    mode: String,
    edition: String,
    option_name: String,
    option_value: String,
) {
    let update: CallbackQuery = cx.update.clone();
    let message: Message = update.message.unwrap();

    let keyboard: InlineKeyboardMarkup = if message.reply_markup().unwrap().inline_keyboard[4][0]
        .text
        .contains("Run")
    {
        keyboards::run_keyboard(version, mode, edition)
    } else {
        keyboards::share_keyboard(version, mode, edition)
    };
    try_join!(
        cx.requester
            .answer_callback_query(update.id)
            .text(format!("set {} to {}", option_name, option_value))
            .send(),
        cx.requester
            .edit_message_reply_markup(message.chat_id(), message.id)
            .reply_markup(keyboard)
            .send()
    )
    .unwrap();
}

/// Run and Share command handler
pub async fn command_handler(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match &command {
        #[allow(unused_variables)]
        Command::Run {
            version,
            mode,
            edition,
        }
        | Command::Share {
            version,
            mode,
            edition,
        } => {
            // Share and Run command need reply message
            if cx.update.reply_to_message().is_some() {
                share_run_answer_message(
                    &cx,
                    &command,
                    version.clone(),
                    mode.clone(),
                    edition.clone(),
                )
                .await?;
            } else {
                cx.reply_to(messages::REPLY_MESSAGE).send().await?;
            };
        }
        _ => (),
    };

    Ok(())
}

pub async fn message_handler(cx: UpdateWithCx<AutoSend<Bot>, Message>) {
    match cx.update.text() {
        Some(text) => {
            if let Some((command, args)) = parse_command(text, bot_username(&cx.requester).await) {
                // Is command
                let command: String = command.to_ascii_lowercase();
                if ["run".into(), "share".into()].contains(&command) {
                    let args: Vec<String> = get_args(args);
                    if command == "run" {
                        command_handler(
                            cx,
                            Command::Run {
                                version: args[0].clone(),
                                mode: args[1].clone(),
                                edition: args[2].clone(),
                            },
                        )
                        .await
                        .unwrap();
                    } else {
                        command_handler(
                            cx,
                            Command::Share {
                                version: args[0].clone(),
                                mode: args[1].clone(),
                                edition: args[2].clone(),
                            },
                        )
                        .await
                        .unwrap();
                    };
                } else if command == "help" {
                    if args.len() > 0 && args[0] == "run" {
                        cx.reply_to(messages::RUN_HELP).send().await.unwrap();
                    } else if args.len() > 0 && args[0] == "share" {
                        cx.reply_to(messages::SHARE_HELP).send().await.unwrap();
                    } else {
                        cx.reply_to(Command::descriptions()).send().await.unwrap();
                    }
                } else if command == "start" {
                    cx.reply_to(messages::START_MESSAGE)
                        .parse_mode(ParseMode::MarkdownV2)
                        .disable_web_page_preview(true)
                        .reply_markup(keyboards::repo_keyboard())
                        .send()
                        .await
                        .unwrap();
                };
            } else {
                // Not command (Text)
            };
        }
        None => (),
    };
}

pub async fn callback_handler(cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>) {
    // callback data be like this in all callback
    // <command> <args> <args> ..
    // viewR <version> <mode> <edition> <already_use_keyboard>
    // viewS <version> <mode> <edition> <already_use_keyboard>
    // already_use
    // print <word>
    // run <version> <mode> <edition>
    // share <version> <mode> <edition>
    // option <version> <mode> <edition> <option_name> <option_value>

    if let Some(callback_data) = cx.update.data.clone() {
        let args: Vec<&str> = callback_data.split_whitespace().collect();
        let command: &str = args[0];
        let args: Vec<&str> = args.into_iter().skip(1).collect();

        match command {
            "viewR" | "viewS" => {
                view_handler(
                    &cx,
                    command,
                    args[0].into(),
                    args[1].into(),
                    args[2].into(),
                    args[3].parse().unwrap(),
                )
                .await
            }
            "already_use" | "print" => {
                cx.requester
                    .answer_callback_query(cx.update.id.clone())
                    .text(if command == "print" {
                        args[0].replace('_', " ")
                    } else {
                        messages::ALREADY_USE_KEYBOARD.to_string()
                    })
                    .send()
                    .await
                    .unwrap();
            }
            "run" | "share" => {
                run_share_callback(&cx, command, args[0].into(), args[1].into(), args[2].into())
                    .await;
            }
            // FIXME: rerror appear when I repeatedly press the inline button ( try it after upgrade to 0.6 )
            "option" => {
                update_options(
                    &cx,
                    args[0].into(),
                    args[1].into(),
                    args[2].into(),
                    args[3].into(),
                    args[4].into(),
                )
                .await
            }
            _ => (),
        };
    }
}

async fn view_handler(
    cx: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    view: &str,
    version: String,
    mode: String,
    edition: String,
    already_use_keyboard: bool,
) {
    let update: CallbackQuery = cx.update.clone();
    if already_use_keyboard {
        already_use_answer(&cx.requester, update.id).await;
    } else {
        // unwrap here because every callback query have message 🙂
        let message: Message = update.message.unwrap();
        let keyboard: InlineKeyboardMarkup = if view == "viewR" {
            keyboards::run_keyboard(version, mode, edition)
        } else {
            keyboards::share_keyboard(version, mode, edition)
        };

        cx.requester
            .edit_message_reply_markup(message.chat_id(), message.id)
            .reply_markup(keyboard)
            .send()
            .await
            .unwrap();
    }
}
