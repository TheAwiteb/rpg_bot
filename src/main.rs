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

#[macro_use]
extern crate diesel;

use dotenv::dotenv;
use teloxide::{dispatching2::UpdateFilterExt, prelude2::*, types::Update};

mod bot;
mod keyboards;
mod messages;
mod models;
mod rpg;
mod rpg_db;
mod schema;

#[tokio::main]
async fn main() {
    dotenv().ok();
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    let bot = Bot::from_env().auto_send();

    log::info!(
        "Starting Rust Playground Bot in https://t.me/{}",
        bot::bot_username(&bot).await
    );

    let handler = dptree::entry()
        // Message branches
        .branch(Update::filter_message().branch(
            dptree::filter(|message: Message| message.text().is_some()).endpoint(
                |message: Message, bot: AutoSend<Bot>| async move {
                    || -> Result<(), ()> {
                        tokio::spawn(bot::message_text_handler(message, bot));
                        Ok(())
                    }()
                },
            ),
        ))
        // Callback query branch
        .branch(Update::filter_callback_query().endpoint(
            |bot: AutoSend<Bot>, callback_query: CallbackQuery| async move {
                || -> Result<(), ()> {
                    tokio::spawn(bot::callback_handler(bot, callback_query));
                    Ok(())
                }()
            },
        ));

    Dispatcher::builder(bot, handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
