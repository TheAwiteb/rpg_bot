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

#[macro_use]
extern crate diesel;

use dotenv::dotenv;
use teloxide::{dispatching::Dispatcher, prelude::*};
use tokio_stream::wrappers::UnboundedReceiverStream;

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
    // TODO: Upgrade teloxide to 0.6 🤩
    teloxide::enable_logging!();
    let bot = Bot::from_env().auto_send();
    let dispatcher: Dispatcher<AutoSend<Bot>> = Dispatcher::new(bot.clone());

    log::info!(
        "Starting Rust Playground Bot in @{}",
        bot::bot_username(&bot).await
    );

    dispatcher
        // message handler
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| async {
            UnboundedReceiverStream::new(rx)
                .for_each_concurrent(None, bot::message_handler)
                .await;
        })
        // callback handler
        .callback_queries_handler(
            |rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| async {
                UnboundedReceiverStream::new(rx)
                    .for_each_concurrent(None, bot::callback_handler)
                    .await;
            },
        )
        .dispatch()
        .await;
}
