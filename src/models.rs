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
    schema::{source_codes, users},
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
        let fullname = format!(
            "{} {}",
            user.first_name,
            user.last_name.clone().unwrap_or(String::new())
        );
        Self::new(user.username.clone(), user.id.to_string(), fullname)
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
                .take(4) // TODO: Use db to get it
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

        // TODO: Use db to get time_limit_expiration
        let time_limit_expiration: i64 = 20; // seconds
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
            .map_err(|err| RpgError::Diesel(err))?;
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
    /// Add attempt to user attempts
    pub async fn make_attempt(&mut self, conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::users::dsl::{attempts, telegram_id, users};
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(attempts.eq(self.attempts + 1))
            .execute(conn)?;
        self.attempts += 1;
        Ok(())
    }

    /// update `last_command_record` (add new record)
    pub async fn make_command_record(&mut self, conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::users::dsl::{last_command_record, telegram_id, users};
        let timestamp = NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0);
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(last_command_record.eq(timestamp))
            .execute(conn)?;
        self.last_command_record = Some(timestamp);
        Ok(()) // The attempt make it in `share_run_answer`
    }

    /// update `last_button_record` (add new record)
    pub async fn make_button_record(&mut self, conn: &mut SqliteConnection) -> DieselResult<()> {
        use super::schema::users::dsl::{last_button_record, telegram_id, users};
        let timestamp = NaiveDateTime::from_timestamp(offset::Utc::now().timestamp(), 0);
        update(users.filter(telegram_id.eq(&self.telegram_id)))
            .set(last_button_record.eq(timestamp))
            .execute(conn)?;
        self.last_button_record = Some(timestamp);
        Ok(()) // The attempt make it in `share_run_answer`
    }

    /// Returns `true` if user can send command to bot
    pub fn can_send_command(&self) -> bool {
        // TODO: Use db to get command_delay
        let command_delay: i64 = 15.into(); // is will get it as `i32` from db
        ((self.last_command_record.is_none())
            || ((self.last_command_record.unwrap().timestamp() + command_delay)
                <= offset::Utc::now().timestamp()))
            && (self.attempts < 100) // TODO: Use db to get attempts
    }

    /// Returns `true` if user can click button
    pub fn can_click_button(&self) -> bool {
        // TODO: Use db to get button_delay
        let button_delay: i64 = 2.into(); // is will get it as `i32` from db
        ((self.last_button_record.is_none())
            || ((self.last_button_record.unwrap().timestamp() + button_delay)
                <= offset::Utc::now().timestamp()))
            && (self.attempts < 100) // TODO: Use db to get attempts
    }

    /// create new source code for user
    pub async fn new_source_code(
        &self,
        conn: &mut SqliteConnection,
        source_code: &Code,
    ) -> DieselResult<SourceCode> {
        NewSourceCode::new(conn, source_code, self)?
            .save(conn)
            .await
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
    pub async fn save(&self, conn: &mut SqliteConnection) -> DieselResult<Users> {
        diesel::insert_into(users::table)
            .values(self)
            .execute(conn)?;
        Ok(Users::try_from((self, conn))?)
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
    pub async fn save(&self, conn: &mut SqliteConnection) -> DieselResult<SourceCode> {
        diesel::insert_into(source_codes::table)
            .values(self)
            .execute(conn)?;

        Ok(SourceCode::try_from((self, conn))?)
    }
}
