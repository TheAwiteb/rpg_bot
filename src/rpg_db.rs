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

use crate::models::{DieselResult, NewUser, Users};
use diesel::prelude::*;
use json_gettext::{static_json_gettext_build, JSONGetText};
use std::env;
use teloxide::types::User as TelegramUser;

/// Returns db connection
pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Returns ctx of languages
pub fn languages_ctx() -> JSONGetText<'static> {
    static_json_gettext_build!(
        "English 🇺🇸";
        "العربية 🇸🇦" => "./i18n/ar_SA.json",
        "English 🇺🇸" => "./i18n/en_US.json",
        "русский 🇷🇺" => "./i18n/ru_RU.json",
    )
    .unwrap()
}

/// Returns old/new user from telegram user object
pub fn get_user(conn: &mut SqliteConnection, author: &TelegramUser) -> DieselResult<Users> {
    Ok(
        Users::try_from((&NewUser::from(author), conn)).unwrap_or_else(|_| {
            NewUser::from(author)
                .save(&mut establish_connection())
                .unwrap()
        }),
    )
}
