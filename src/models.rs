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

use crate::schema::users::attempts;

use super::schema::{source_codes, users};
use chrono::{offset, NaiveDateTime};
use diesel::{prelude::*, update};
use teloxide::types::User as TelegramUser;

#[derive(Queryable)]
pub struct Users {
    pub id: i32,
    pub username: Option<String>,
    pub telegram_id: String,
    pub telegram_fullname: String,
    pub attempts: i32,
    pub last_record: Option<NaiveDateTime>,
}

#[derive(Queryable)]
pub struct SourceCode {
    pub id: i32,
    pub user_id: i32,
    pub source_code: String,
    pub code: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: Option<String>,
    pub telegram_id: String,
    pub telegram_fullname: String,
}

#[derive(Insertable)]
#[table_name = "source_codes"]
pub struct NewSourceCode {
    pub user_id: i32,
    pub source_code: String,
    pub code: String,
    pub created_at: NaiveDateTime,
}

impl From<(&NewSourceCode, &mut SqliteConnection)> for SourceCode {
    fn from((source, conn): (&NewSourceCode, &mut SqliteConnection)) -> SourceCode {
        use super::schema::source_codes::dsl::{code, source_codes};
        source_codes
            .filter(code.eq(source.code.clone()))
            .first::<SourceCode>(conn)
            .expect(&format!("Source with '{}' code not found!", source.code))
    }
}

impl From<(&NewUser, &mut SqliteConnection)> for Users {
    fn from((user, conn): (&NewUser, &mut SqliteConnection)) -> Users {
        use super::schema::users::dsl::{telegram_id, users};
        users
            .filter(telegram_id.eq(&user.telegram_id))
            .first::<Users>(conn)
            .expect(&format!("No user with '{}' telegram id", user.telegram_id))
    }
}

impl From<TelegramUser> for NewUser {
    /// Returns new user object from telegram user
    fn from(user: TelegramUser) -> Self {
        let fullname = format!(
            "{} {}",
            user.first_name,
            user.last_name.unwrap_or(String::new())
        );
        Self::new(user.username, format!("{}", user.id), fullname)
    }
}

impl SourceCode {
    fn code() -> String {
        String::new()
    }

    /// Returns source code by code
    pub fn get_by_code(code: &str, conn: &mut SqliteConnection) -> Option<Self> {
        use super::schema::source_codes::dsl::{code as source_code, source_codes};
        source_codes
            .filter(source_code.eq(code))
            .first::<SourceCode>(conn)
            .ok()
    }

    /// Returns source author
    pub fn author(&self, conn: &mut SqliteConnection) -> Users {
        use super::schema::users::dsl::{id, users};
        users
            .filter(id.eq(&self.id))
            .first::<Users>(conn)
            .expect(&format!("No user with '{}' id", self.id))
    }
}

impl Users {
    /// Add attempt to user attempts
    pub fn make_attempt(&mut self, conn: &mut SqliteConnection) {
        use super::schema::users::dsl::{telegram_id, users};
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(attempts.eq(self.attempts + 1))
            .execute(conn)
            .expect("failed to update {} attempts");
        self.attempts += 1;
    }

    /// update `last_record` (add new record)
    pub fn make_record(&mut self, conn: &mut SqliteConnection) {
        use super::schema::users::dsl::{last_record, telegram_id, users};
        let timestamp = NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0);
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(last_record.eq(timestamp))
            .execute(conn)
            .expect("failed to update {} attempts");
        self.last_record = Some(timestamp);
    }
}

impl NewUser {
    /// Make new object, you can save it in database use save method
    pub fn new(username: Option<String>, telegram_id: String, telegram_fullname: String) -> Self {
        Self {
            username,
            telegram_id,
            telegram_fullname,
        }
    }

    /// save object in database
    pub fn save(&self, conn: &mut SqliteConnection) -> Users {
        diesel::insert_into(users::table)
            .values(self)
            .execute(conn)
            .expect("Error saving new source");
        Users::from((self, conn))
    }
}

impl NewSourceCode {
    /// Make new object, you can save it in database use save method
    pub fn new(source_code: String, author: Users) -> Self {
        Self {
            source_code,
            code: SourceCode::code(),
            user_id: author.id as i32,
            created_at: NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0),
        }
    }

    /// save object in database
    pub fn save(&self, conn: &mut SqliteConnection) -> SourceCode {
        diesel::insert_into(source_codes::table)
            .values(self)
            .execute(conn)
            .expect("Error saving new source");
        SourceCode::from((self, conn))
    }
}
