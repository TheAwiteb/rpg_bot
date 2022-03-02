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

pub const RUN_HELP: &str = r#"Reply to message with this command to execute Rust code 🦀⚙️
    /run <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>
    Example:
        /run stable debug 2021"#;
pub const SHARE_HELP: &str = r#"Reply to message with this command to share Rust code 🦀🔗
    /share <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>
    Example:
        /share stable debug 2021"#;
pub const START_MESSAGE: &str = "Welcome, with this bot you can run and share rust code with [Rust Playground](https://play.rust-lang.org/)\nfor help message type /help";
pub const REPLY_MESSAGE: &str = "Use this command in a reply to another message!";
pub const MESSAGE_CANNOT_REACHED: &str = "The message with the source code cannot be reached ❗";
pub const ALREADY_USE_KEYBOARD: &str = "The source code is already has run/share it";
pub const CANNOT_RUN_INVALID_CODE: &str = "Cannot_Run_invalid_source_code_🤨";
pub const CANNOT_SHARE_INVALID_CODE: &str = "Cannot_share_invalid_source_code_🤨";
pub const MUST_BE_TEXT: &str = "The source code must be text ❗";
