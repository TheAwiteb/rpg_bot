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

use crate::{models::SourceCode, rpg_db::languages_ctx};
use json_gettext::get_text;
use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn repo_keyboard(language: &str) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        get_text!(ctx, language, "REPOSITORY").unwrap().to_string() + " 🦀",
        Url::parse("https://github.com/TheAwiteb/rpg_bot").unwrap(),
    )]])
}

fn option_keyboard(
    version: &str,
    mode: &str,
    edition: &str,
    code: &str,
    language: &str,
) -> InlineKeyboardMarkup {
    // keyboard will be like this
    //
    // Version 📦 | Mode ​🚀​   | Edition ​⚡
    //  Stable  ⬅️ | Debug   ⬅️ | 2015 -
    //  Beta    - | Release - | 2018 -
    //  Nightly - | _         | 2021 ⬅️
    //
    let check = "⬅️";
    let uncheck = "-";
    let ctx = languages_ctx();

    let mut keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback(
            get_text!(ctx, language, "VERSION").unwrap().to_string() + " 📦\u{200B}",
            format!(
                "print {}",
                get_text!(ctx, language, "VERSION_OF_CODE").unwrap()
            ) + "_📦\u{200B}",
        ),
        InlineKeyboardButton::callback(
            get_text!(ctx, language, "MODE").unwrap().to_string() + " 🚀",
            format!(
                "print {}",
                get_text!(ctx, language, "MODE_OF_CODE").unwrap()
            ) + "_🚀\u{200B}",
        ),
        InlineKeyboardButton::callback(
            get_text!(ctx, language, "EDITION").unwrap().to_string() + " ⚡\u{200B}",
            format!(
                "print {}",
                get_text!(ctx, language, "EDITION_OF_CODE").unwrap()
            ) + "_⚡",
        ),
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

pub fn add_lang_keyboard(language: &str) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();
    InlineKeyboardMarkup::new([[InlineKeyboardButton::url(
        get_text!(ctx, language, "ADD_NEW_LANGUAGE")
            .unwrap()
            .to_string()
            + " 🤩",
        Url::parse("https://github.com/TheAwiteb/rpg_bot#Add-new-language").unwrap(),
    )]])
}

pub fn view_run_keyboard(
    code: impl AsRef<str>,
    already_use_keyboard: bool,
    is_valid_source: bool,
    language: &str,
) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        get_text!(ctx, language, "RUN").unwrap().to_string() + " 🦀⚙️",
        if is_valid_source {
            // if source code is valid, the code will be valid
            format!("viewR {} {}", code.as_ref(), already_use_keyboard)
        } else {
            format!(
                "print {}",
                get_text!(ctx, language, "CANNOT_RUN_INVALID_CODE").unwrap()
            )
        },
    )]])
}

pub fn view_share_keyboard(
    code: impl AsRef<str>,
    already_use_keyboard: bool,
    is_valid_source: bool,
    language: &str,
) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();

    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        get_text!(ctx, language, "SHARE").unwrap().to_string() + " 🦀🔗",
        if is_valid_source {
            // if source code is valid, the code will be valid
            format!("viewS {} {}", code.as_ref(), already_use_keyboard)
        } else {
            format!(
                "print {}",
                get_text!(ctx, language, "CANNOT_SHARE_INVALID_CODE").unwrap()
            )
        },
    )]])
}

pub fn run_keyboard(source: SourceCode, language: &str) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();

    option_keyboard(
        &source.version,
        &source.mode,
        &source.edition,
        &source.code,
        language,
    )
    .append_row([InlineKeyboardButton::callback(
        get_text!(ctx, language, "RUN").unwrap().to_string() + " 🦀⚙️",
        format!("run {}", source.code),
    )])
}

pub fn share_keyboard(source: SourceCode, language: &str) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();

    option_keyboard(
        &source.version,
        &source.mode,
        &source.edition,
        &source.code,
        language,
    )
    .append_row([InlineKeyboardButton::callback(
        get_text!(ctx, language, "SHARE").unwrap().to_string() + " 🦀🔗",
        format!("share {}", source.code),
    )])
}
pub fn languages_keyboard(language: &str) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();

    InlineKeyboardMarkup::new(
        ctx.get_keys()
            .into_iter()
            .map(|lang: &str| {
                InlineKeyboardButton::callback(
                    format!("{}{}", if language == lang { "🌟 " } else { "" }, lang),
                    format!("change_lang {}", lang.replace(" ", "_")),
                )
            })
            .collect::<Vec<InlineKeyboardButton>>()
            .chunks(2)
            .map(|row: &[InlineKeyboardButton]| row.to_vec()),
    )
    .append_row(add_lang_keyboard(language).inline_keyboard[0].clone())
}
