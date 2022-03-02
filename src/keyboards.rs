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

use crate::{messages, models::SourceCode};
use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn repo_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "Repository 🦀".to_string(),
        Url::parse("https://github.com/TheAwiteb/rpg_bot").unwrap(),
    )]])
}

fn option_keyboard(version: &str, mode: &str, edition: &str, code: &str) -> InlineKeyboardMarkup {
    // keyboard will be like this
    //
    // Version 📦 | Mode ​🚀​   | Edition ​⚡
    //  Stable  ⬅️ | Debug   ⬅️ | 2015 -
    //  Beta    - | Release - | 2018 -
    //  Nightly - | _         | 2021 ⬅️
    //
    let check = "⬅️";
    let uncheck = "-";

    let mut keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Version 📦".into(), "print Version_of_code_📦".into()),
        InlineKeyboardButton::callback("Mode ​🚀​".into(), "print Mode_of_code_🚀".into()),
        InlineKeyboardButton::callback("Edition ​⚡​".into(), "print Edition_of_code_⚡".into()),
    ]]);
    let buttons: [&str; 9] = [
        "Stable", "Debug", "2015", "Beta", "Release", "2018", "Nightly", "-", "2021",
    ];
    for row in buttons.chunks(3) {
        keyboard = keyboard.append_row(row.iter().enumerate().map(|(idx, button)| {
            let args: Vec<&str> = vec![version, mode, edition];
            let it_same: bool = button.to_lowercase() == args[idx];
            InlineKeyboardButton::callback(
                format!("{} {}", button, if it_same { check } else { uncheck }),
                if button == &"-" {
                    "print 😑".into()
                } else {
                    format!(
                        "option {} {} {}",
                        code,
                        match idx {
                            0 => "version",
                            1 => "mode",
                            _ => "edition",
                        },
                        button.to_lowercase()
                    )
                },
            )
        }));
    }
    keyboard
}

pub fn view_run_keyboard(
    code: Option<String>,
    already_use_keyboard: bool,
    is_valid_source: bool,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "Run 🦀⚙️".into(),
        if is_valid_source {
            // if source code is valid, the code will be valid
            format!("viewR {} {}", code.unwrap(), already_use_keyboard)
        } else {
            format!("print {}", messages::CANNOT_RUN_INVALID_CODE)
        },
    )]])
}

pub fn view_share_keyboard(
    code: Option<String>,
    already_use_keyboard: bool,
    is_valid_source: bool,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "Share 🦀🔗".into(),
        if is_valid_source {
            // if source code is valid, the code will be valid
            format!("viewS {} {}", code.unwrap(), already_use_keyboard)
        } else {
            format!("print {}", messages::CANNOT_SHARE_INVALID_CODE)
        },
    )]])
}

pub fn run_keyboard(source: SourceCode) -> InlineKeyboardMarkup {
    option_keyboard(&source.version, &source.mode, &source.edition, &source.code).append_row([
        InlineKeyboardButton::callback("Run 🦀⚙️".into(), format!("run {}", source.code)),
    ])
}

pub fn share_keyboard(source: SourceCode) -> InlineKeyboardMarkup {
    option_keyboard(&source.version, &source.mode, &source.edition, &source.code).append_row([
        InlineKeyboardButton::callback("Share 🦀🔗".into(), format!("share {}", source.code)),
    ])
}
