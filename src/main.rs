use dotenv::dotenv;
use teloxide::{dispatching::Dispatcher, prelude::*};
use tokio_stream::wrappers::UnboundedReceiverStream;

mod bot;
mod keyboards;
mod messages;
mod rpg;

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
