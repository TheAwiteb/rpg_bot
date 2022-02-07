pub const RUN_HELP: &str = r#"Reply to message with this command to execute Rust code 🦀⚙️
    /run <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>
    Example:
        /run stable debug 2021"#;
pub const SHARE_HELP: &str = r#"Reply to message with this command to share Rust code 🦀🔗
    /share <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>
    Example:
        /share stable debug 2021"#;
pub const START_MESSAGE: &str = "Welcome, with this bot you can run and share rust code with [Rust Playground](https://play.rust-lang.org/)\nfor help message type /help";
pub const REPLY_MESSAGE: &str = "Use this command in a reply to another message!";
pub const MESSAGE_CANNOT_REACHED: &str = "The message with the source code cannot be reached ❗";
pub const ALREADY_USE_KEYBOARD: &str = "The source code is already has run/share it";
