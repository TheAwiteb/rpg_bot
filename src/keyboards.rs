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

use std::num::TryFromIntError;

use crate::{
    models::{Config, SourceCode, Users},
    rpg_db::{self, languages_ctx},
};
use diesel::SqliteConnection;
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
                    format!("change_lang {}", lang.replace(' ', "_")),
                )
            })
            .collect::<Vec<InlineKeyboardButton>>()
            .chunks(2)
            .map(|row: &[InlineKeyboardButton]| row.to_vec()),
    )
    .append_row(add_lang_keyboard(language).inline_keyboard[0].clone())
}

pub fn admin_main_keybard(language: &str) -> InlineKeyboardMarkup {
    let ctx = languages_ctx();

    InlineKeyboardMarkup::new([
        vec![
            InlineKeyboardButton::callback(
                format!("{} 👤", get_text!(ctx, language, "USERS").unwrap()),
                "goto users".into(),
            ),
            InlineKeyboardButton::callback(
                format!("{} ⚙️", get_text!(ctx, language, "SETTINGS").unwrap()),
                "goto settings".into(), // TODO
            ),
        ],
        vec![InlineKeyboardButton::callback(
            format!("{} 🔈", get_text!(ctx, language, "BROADCAST").unwrap()),
            "goto broadcast".into(), // TODO
        )],
    ])
}

/// Returns users interface
pub fn admin_users_keybard(
    conn: &mut SqliteConnection,
    user_telegram_id: i64,
    language: &str,
    page_number: u32,
) -> Result<InlineKeyboardMarkup, TryFromIntError> {
    let ctx = languages_ctx();
    // default value is 10
    let users_in_page: u32 = Config::get_or_add("user_in_users_page", "10", conn)
        .value
        .parse::<u32>()
        .expect("`command_delay` config should be unsigned integer");
    let users = Users::all_users(conn).unwrap_or_default().into_iter();
    let maximum_users_in_page: usize = ((page_number + 1) * users_in_page) as usize;
    let users_count = users.len();

    Ok(InlineKeyboardMarkup::new(
        vec![vec![
            InlineKeyboardButton::callback(
                get_text!(ctx, language, "USER_INFO").unwrap().to_string() + " 👤",
                format!(
                    "print {}",
                    get_text!(ctx, language, "USER_INFO_ANSWER")
                        .unwrap()
                        .to_string()
                        .replace(' ', "_")
                ) + "_👤",
            ),
            InlineKeyboardButton::callback(
                get_text!(ctx, language, "BANNED").unwrap().to_string() + " 🚫",
                format!(
                    "print {}",
                    get_text!(ctx, language, "BANNED_STATUS")
                        .unwrap()
                        .to_string()
                        .replace(' ', "_")
                        + "_🚫"
                ),
            ),
            InlineKeyboardButton::callback(
                get_text!(ctx, language, "ADMINISTRATIVE")
                    .unwrap()
                    .to_string()
                    + " 👮‍♂️",
                format!(
                    "print {}",
                    get_text!(ctx, language, "ADMIN_STATUS")
                        .unwrap()
                        .to_string()
                        .replace(' ', "_")
                ) + "_👮‍♂️",
            ),
        ]]
        .into_iter()
        .chain(
            if users_count.ge(&maximum_users_in_page) {
                users
                    .enumerate()
                    // example if page number is 2:
                    // Take 10 users from (20 - 10) to (20)
                    // Note: 10 is default users in one page
                    // 20 is maximum users in two page
                    .take_while(|(idx, _)| {
                        idx.ge(&(maximum_users_in_page + users_in_page as usize))
                            && idx.le(&maximum_users_in_page)
                    })
                    .map(|(_, user)| user)
                    .collect::<Vec<Users>>()
            } else if (isize::try_from(users_count)? - isize::try_from(users_in_page)?).lt(&0) {
                users.collect()
            } else {
                users
                    .enumerate()
                    // Take last 10 users (While index >= users_count - 10)
                    // Note: 10 is default users in one page
                    .take_while(|(idx, _)| idx.ge(&(users_count - users_in_page as usize)))
                    .map(|(_, user)| user)
                    .collect()
            }
            .into_iter()
            .map(|user| {
                vec![
                    InlineKeyboardButton::callback(
                        user.telegram_fullname.clone(),
                        // TODO
                        format!("goto users-info {} {}", user.telegram_id, page_number),
                    ),
                    InlineKeyboardButton::callback(
                        if user.is_ban { "✔️" } else { "✖️" }.to_string(),
                        // TODO
                        format!("admin users ban {} {}", user.telegram_id, page_number),
                    ),
                    InlineKeyboardButton::callback(
                        if user.is_admin { "✔️" } else { "✖️" }.to_string(),
                        if user_telegram_id.eq(&(rpg_db::super_user_id() as i64)) {
                            if user.telegram_id.ne(&user_telegram_id.to_string()) {
                                // TODO
                                format!("admin users admin {} {}", user.telegram_id, page_number)
                            } else {
                                format!(
                                    "print {}",
                                    get_text!(ctx, language, "CANNOT_UNADMIN_YORSELF")
                                        .unwrap()
                                        .to_string()
                                        .replace(' ', "_")
                                )
                            }
                        } else {
                            format!(
                                "print {}",
                                get_text!(ctx, language, "SUPER_USER_COMMAND_ERROR")
                                    .unwrap()
                                    .to_string()
                                    .replace(' ', "_")
                            )
                        },
                    ),
                ]
            }),
        )
        .chain(
            // Back button (To main admin interface)
            vec![vec![InlineKeyboardButton::callback(
                format!("{} 🔙", get_text!(ctx, language, "BACK_BUTTON").unwrap()),
                "goto admin".to_string(),
            )]],
        ),
    ))
}
