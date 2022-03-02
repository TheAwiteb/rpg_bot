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

table! {
    config (id) {
        id -> Integer,
        name -> Text,
        value -> Text,
    }
}

table! {
    source_codes (id) {
        id -> Integer,
        user_id -> Integer,
        code -> Text,
        source_code -> Text,
        version -> Text,
        edition -> Text,
        mode -> Text,
        created_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Nullable<Text>,
        telegram_id -> Text,
        telegram_fullname -> Text,
        attempts -> Integer,
        attempts_maximum -> Integer,
        last_command_record -> Nullable<Timestamp>,
        last_button_record -> Nullable<Timestamp>,
    }
}

joinable!(source_codes -> users (user_id));

allow_tables_to_appear_in_same_query!(config, source_codes, users,);
