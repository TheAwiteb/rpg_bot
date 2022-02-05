use teloxide::prelude::*;

use dotenv::dotenv;

mod bot;
mod messages;
mod rpg;

#[tokio::main]
async fn main() {
    dotenv().ok();
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    let bot = Bot::from_env().auto_send();

    log::info!(
        "Starting Rust Playground Bot in @{}",
        bot::bot_username(&bot).await
    );

    teloxide::repl(bot.clone(), bot::main_handler).await;
}
