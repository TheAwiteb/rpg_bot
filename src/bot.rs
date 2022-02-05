use crate::{messages, rpg, keyboards};
use std::error::Error;
use teloxide::payloads::SendMessageSetters;
use teloxide::utils::command::parse_command;
use teloxide::{prelude::*, requests::Requester, types::ParseMode, utils::command::BotCommand};

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

/// Returns bot username
pub async fn bot_username(bot: &AutoSend<Bot>) -> String {
    bot.get_me()
        .await
        .unwrap()
        .user
        .username
        .expect("Bots must have usernames")
}

/// Send wait message for command
async fn send_wait_message(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    command: &Command,
) -> Result<Message, Box<dyn Error + Send + Sync>> {
    let message = match &command {
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
    };

    Ok(cx.reply_to(&message).send().await?)
}

/// Send code output for run command and Rust playground for share command
#[allow(unused_variables)]
pub async fn share_run_answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: &Command,
    version: String,
    mode: String,
    edition: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let code: rpg::Code = rpg::Code {
        source_code: cx
            .update
            .reply_to_message()
            .unwrap()
            .text()
            .unwrap()
            .to_owned(),
        version,
        mode,
        edition,
    };

    if let Err(err) = code.is_valid() {
        cx.reply_to(err).send().await?;
    } else {
        let message: Message = send_wait_message(&cx, &command).await?;
        let output: String = if matches!(
            command,
            Command::Run {
                version,
                mode,
                edition
            }
        ) {
            rpg::run(code).await?
        } else {
            rpg::share(code).await?
        };

        cx.requester
            .edit_message_text(
                // For text messages, the actual UTF-8 text of the message, 0-4096 characters
                // https://core.telegram.org/bots/api#message
                message.chat_id(),
                message.id,
                &output[..if output.chars().count() > 4096 {
                    4096
                } else {
                    output.chars().count()
                }],
            )
            .send()
            .await?;
    }
    Ok(())
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
                share_run_answer(cx, &command, version.clone(), mode.clone(), edition.clone())
                    .await?;
            } else {
                cx.reply_to(messages::REPLY_MESSAGE).send().await?;
            };
        }
        _ => (),
    };

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

pub async fn main_handler(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.text() {
        Some(text) => {
            if let Some((command, args)) = parse_command(text, bot_username(&cx.requester).await) {
                // Is command
                let command = command.to_ascii_lowercase();
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
                        .await?;
                    } else {
                        command_handler(
                            cx,
                            Command::Share {
                                version: args[0].clone(),
                                mode: args[1].clone(),
                                edition: args[2].clone(),
                            },
                        )
                        .await?;
                    };
                } else if command == "help" {
                    if args.len() > 0 && args[0] == "run" {
                        cx.reply_to(messages::RUN_HELP).send().await?;
                    } else if args.len() > 0 && args[0] == "share" {
                        cx.reply_to(messages::SHARE_HELP).send().await?;
                    } else {
                        cx.reply_to(Command::descriptions()).send().await?;
                    }
                } else if command == "start" {
                    cx.reply_to(messages::START_MESSAGE)
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
        None => (),
    };

    Ok(())
}
