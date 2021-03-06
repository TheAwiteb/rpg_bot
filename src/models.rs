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

use super::{
    rpg::Code,
    schema::{config, source_codes, users},
};
use chrono::{offset, NaiveDateTime};
use diesel::{prelude::*, query_builder::UpdateStatement, update};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use teloxide::types::User as TelegramUser;

#[derive(Queryable)]
pub struct Users {
    pub id: i32,
    pub username: Option<String>,
    pub telegram_id: String,
    pub telegram_fullname: String,
    pub language: String,
    pub attempts: i32,
    pub attempts_maximum: i32,
    pub last_command_record: Option<NaiveDateTime>,
    pub last_button_record: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable)]
pub struct SourceCode {
    pub id: i32,
    pub user_id: i32,
    pub code: String,
    pub source_code: String,
    pub version: String,
    pub edition: String,
    pub mode: String,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable)]
pub struct Config {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: Option<String>,
    pub telegram_id: String,
    pub telegram_fullname: String,
}

#[derive(Debug, Insertable)]
#[table_name = "source_codes"]
pub struct NewSourceCode {
    pub user_id: i32,
    pub code: String,
    pub source_code: String,
    pub version: String,
    pub edition: String,
    pub mode: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "config"]
pub struct NewConfig {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub enum RpgError {
    Diesel(DieselError),
    Text(String),
}

pub type DieselError = diesel::result::Error;
pub type DieselResult<T> = Result<T, DieselError>;

impl TryFrom<(&NewSourceCode, &mut SqliteConnection)> for SourceCode {
    type Error = DieselError;

    fn try_from((source, conn): (&NewSourceCode, &mut SqliteConnection)) -> DieselResult<Self> {
        use super::schema::source_codes::dsl::{code, source_codes};
        source_codes
            .filter(code.eq(source.code.clone()))
            .first::<Self>(conn)
    }
}

impl TryFrom<(&NewUser, &mut SqliteConnection)> for Users {
    type Error = DieselError;

    fn try_from((user, conn): (&NewUser, &mut SqliteConnection)) -> DieselResult<Self> {
        use super::schema::users::dsl::{telegram_id, users};
        users
            .filter(telegram_id.eq(&user.telegram_id))
            .first::<Self>(conn)
    }
}

impl From<&TelegramUser> for NewUser {
    /// Returns new user object from telegram user
    fn from(user: &TelegramUser) -> Self {
        Self::new(user.username.clone(), user.id.to_string(), user.full_name())
    }
}

impl From<SourceCode> for NewSourceCode {
    fn from(source: SourceCode) -> Self {
        Self {
            user_id: source.user_id,
            code: source.code,
            source_code: source.source_code,
            version: source.version,
            mode: source.mode,
            edition: source.edition,
            created_at: source.created_at,
        }
    }
}

impl NewConfig {
    fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl Config {
    /// Get config by name
    pub fn get_by_name(name: &str, conn: &mut SqliteConnection) -> Option<Self> {
        use super::schema::config::dsl::{config, name as name_};
        config.filter(name_.eq(name)).first::<Self>(conn).ok()
    }

    /// Returns config with `name` if exist or add new to db with `name` and `value`
    pub fn get_or_add(name: &str, value: &str, conn: &mut SqliteConnection) -> Self {
        Config::get_by_name(name, conn).unwrap_or_else(|| Config::add(name, value, conn).unwrap())
    }

    /// Add new config to db
    pub fn add(name: &str, value: &str, conn: &mut SqliteConnection) -> DieselResult<Self> {
        diesel::insert_into(config::table)
            .values(NewConfig::new(name, value))
            .execute(conn)?;
        Ok(Config::get_by_name(name, conn).unwrap())
    }
}

impl SourceCode {
    /// Returns `true` if the code are exist in database
    pub fn code_is_exist(conn: &mut SqliteConnection, code: &str) -> bool {
        use super::schema::source_codes::dsl::{code as code_, source_codes};
        source_codes
            .filter(code_.eq(&code))
            .first::<Self>(conn)
            .ok()
            .is_some()
    }

    /// Returns new code that not in database
    /// The code is a distinctive code that distinguishes the source code from others, (it is used to request it instead of using id)
    pub fn code(conn: &mut SqliteConnection) -> DieselResult<String> {
        loop {
            // create random code
            let code: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(
                    // default value is 4
                    Config::get_or_add("code_length", "4", conn)
                        .value
                        .parse::<usize>()
                        .expect("`code_length` config should be integer"),
                )
                .map(char::from)
                .collect::<String>()
                .to_ascii_lowercase();

            if !Self::code_is_exist(conn, &code) {
                return Ok(code);
            }
        }
    }

    /// Use this function to remove all source codes that have expired
    pub fn filter_source_codes(conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::source_codes::dsl::{created_at, source_codes};

        // default value is one week
        let time_limit_expiration: i64 = Config::get_or_add(
            "time_limit_expiration",
            &(((60 * 60/* One houre */) * 24/* One day */) * 7/* One week */).to_string(),
            conn,
        )
        .value
        .parse::<i64>()
        .expect("`time_limit_expiration` config should be integer"); // in seconds
        diesel::delete(
            source_codes.filter(created_at.le(NaiveDateTime::from_timestamp(
                offset::Utc::now().timestamp() - time_limit_expiration,
                0,
            ))),
        )
        .execute(conn)?;
        Ok(())
    }

    /// Returns source code by code
    pub fn get_by_code(code: &str, conn: &mut SqliteConnection) -> DieselResult<Self> {
        use super::schema::source_codes::dsl::{code as code_, source_codes};
        source_codes.filter(code_.eq(code)).first::<Self>(conn)
    }

    /// Returns source author
    pub fn author(&self, conn: &mut SqliteConnection) -> DieselResult<Users> {
        use super::schema::users::dsl::{id, users};
        users.filter(id.eq(&self.user_id)).first::<Users>(conn)
    }

    // Update field by name, just can update `version`, `edition`, `mode`
    pub fn update_by_name(
        &mut self,
        field_name: &str,
        new_value: &str,
        conn: &mut SqliteConnection,
    ) -> Result<(), RpgError> {
        use super::schema::source_codes::dsl::{edition, mode, source_codes, version};

        if ["version", "edition", "mode"].contains(&field_name) {
            let update_statement: UpdateStatement<_, _> = update(source_codes.find(self.id));
            match field_name {
                "version" => {
                    self.version = new_value.into();
                    update_statement.set(version.eq(new_value)).execute(conn)
                }
                "edition" => {
                    self.edition = new_value.into();
                    update_statement.set(edition.eq(new_value)).execute(conn)
                }
                _ => {
                    self.mode = new_value.into();
                    update_statement.set(mode.eq(new_value)).execute(conn)
                }
            }
            .map_err(RpgError::Diesel)?;
            Ok(())
        } else {
            Err(RpgError::Text(format!(
                "Cannot update {} field",
                field_name
            )))
        }
    }
}

impl Users {
    /// Update user (`username` and `telegram_fullname`)
    pub async fn update(
        &mut self,
        user: &TelegramUser,
        conn: &mut SqliteConnection,
    ) -> DieselResult<()> {
        if self.username != user.username {
            self.update_username(&user.username, conn)?;
        }
        if self.telegram_fullname != user.full_name() {
            self.update_telegram_fullname(user.full_name().as_ref(), conn)?;
        };

        Ok(())
    }

    /// Add attempt to user attempts
    pub fn make_attempt(&mut self, conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::users::dsl::{attempts, telegram_id, users};
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(attempts.eq(self.attempts + 1))
            .execute(conn)?;
        self.attempts += 1;
        Ok(())
    }

    /// update `last_command_record` (add new record)
    pub fn make_command_record(&mut self, conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::users::dsl::{last_command_record, telegram_id, users};
        let timestamp = NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0);
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(last_command_record.eq(timestamp))
            .execute(conn)?;
        self.last_command_record = Some(timestamp);
        Ok(()) // The attempt make it in `share_run_answer`
    }

    /// update `last_button_record` (add new record)
    pub fn make_button_record(&mut self, conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::users::dsl::{last_button_record, telegram_id, users};
        let timestamp = NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0);
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(last_button_record.eq(timestamp))
            .execute(conn)?;
        self.last_button_record = Some(timestamp);
        Ok(()) // The attempt make it in `share_run_answer`
    }

    /// update `language`
    pub fn update_language(
        &mut self,
        new_language: &str,
        conn: &mut SqliteConnection,
    ) -> DieselResult<()> {
        use super::schema::users::dsl::{language, users};
        update(users.find(self.id))
            .set(language.eq(new_language))
            .execute(conn)?;
        self.language = new_language.into();
        Ok(())
    }

    /// update `telegram_fullname`
    pub fn update_telegram_fullname(
        &mut self,
        new_telegram_fullname: &str,
        conn: &mut SqliteConnection,
    ) -> DieselResult<()> {
        use super::schema::users::dsl::{telegram_fullname, users};
        update(users.find(self.id))
            .set(telegram_fullname.eq(new_telegram_fullname))
            .execute(conn)?;
        self.telegram_fullname = new_telegram_fullname.into();
        Ok(())
    }

    /// update `username`
    pub fn update_username(
        &mut self,
        new_username: &Option<String>,
        conn: &mut SqliteConnection,
    ) -> DieselResult<()> {
        use super::schema::users::dsl::{username, users};
        update(users.find(self.id))
            .set(username.eq(new_username))
            .execute(conn)?;
        self.username = new_username.clone();
        Ok(())
    }

    /// Returns `true` if user can send command to bot
    pub fn can_send_command(&self, conn: &mut SqliteConnection) -> bool {
        let command_delay: i64 = // default value is 15
        Config::get_or_add("command_delay", "15", conn)
            .value
            .parse::<i64>()
            .expect("`command_delay` config should be integer");
        ((self.last_command_record.is_none())
            || ((self.last_command_record.unwrap().timestamp() + command_delay)
                <= offset::Utc::now().timestamp()))
            && (self.attempts < self.attempts_maximum)
    }

    /// Returns `true` if user can click button
    pub fn can_click_button(&self, conn: &mut SqliteConnection) -> bool {
        let button_delay: i64 = // default value is 2
        Config::get_or_add("button_delay", "2", conn)
            .value
            .parse::<i64>()
            .expect("`button_delay` config should be integer");
        ((self.last_button_record.is_none())
            || ((self.last_button_record.unwrap().timestamp() + button_delay)
                <= offset::Utc::now().timestamp()))
            && (self.attempts < self.attempts_maximum)
    }

    /// create new source code for user
    pub fn new_source_code(
        &self,
        conn: &mut SqliteConnection,
        source_code: &Code,
    ) -> DieselResult<SourceCode> {
        NewSourceCode::new(conn, source_code, self)?.save(conn)
    }

    /// Returns source codes of user
    pub fn source_codes(&self, conn: &mut SqliteConnection) -> Option<Vec<SourceCode>> {
        use super::schema::source_codes::dsl::{source_codes, user_id};
        if let Ok(sources) = source_codes
            .filter(user_id.eq(self.id))
            .load::<SourceCode>(conn)
        {
            Some(sources)
        } else {
            None
        }
    }
}

impl NewUser {
    /// Make new object, you can save it in database use save method
    pub fn new<T: Into<String>>(
        username: Option<String>,
        telegram_id: T,
        telegram_fullname: T,
    ) -> Self {
        Self {
            username,
            telegram_id: telegram_id.into(),
            telegram_fullname: telegram_fullname.into(),
        }
    }

    /// save object in database
    pub fn save(&self, conn: &mut SqliteConnection) -> DieselResult<Users> {
        diesel::insert_into(users::table)
            .values(self)
            .execute(conn)?;
        Users::try_from((self, conn))
    }
}

impl NewSourceCode {
    /// Make new object, you can save it in database use save method
    pub fn new(
        conn: &mut SqliteConnection,
        source_code: &Code,
        author: &Users,
    ) -> DieselResult<Self> {
        Ok(Self {
            source_code: source_code.source_code.to_string(),
            version: source_code.version.to_string(),
            edition: source_code.edition.to_string(),
            mode: source_code.mode.to_string(),
            code: SourceCode::code(conn)?,
            user_id: author.id as i32,
            created_at: NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0),
        })
    }

    /// save object in database
    pub fn save(&self, conn: &mut SqliteConnection) -> DieselResult<SourceCode> {
        if !SourceCode::code_is_exist(conn, &self.code) {
            diesel::insert_into(source_codes::table)
                .values(self)
                .execute(conn)?;
        }

        SourceCode::try_from((self, conn))
    }
}
