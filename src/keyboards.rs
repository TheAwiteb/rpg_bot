use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn repo_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "Repository 🦀".to_string(),
        "https://github.com/TheAwiteb/rpg_bot".to_string(),
    )]])
}

fn option_keyboard(version: &str, mode: &str, edition: &str) -> InlineKeyboardMarkup {
    // keyboard will be like this
    //
    // Version 📦 | Mode ​🚀​   | Edition ​⚡
    //  Stable  ⬅️ | Debug   ⬅️ | 2015 -
    //  Beta    - | Release - | 2018 -
    //  Nightly - | _         | 2021 ⬅️
    //
    let check = "⬅️";
    let uncheck = "-";

    let mut keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Version 📦".into(), "print Version_of_code_📦".into()),
        InlineKeyboardButton::callback("Mode ​🚀​".into(), "print Mode_of_code_🚀".into()),
        InlineKeyboardButton::callback("Edition ​⚡​".into(), "print Edition_of_code_⚡".into()),
    ]]);
    let buttons: [&str; 9] = [
        "Stable", "Debug", "2015", "Beta", "Release", "2018", "Nightly", "-", "2021",
    ];
    for row in buttons.chunks(3) {
        keyboard = keyboard.append_row(row.iter().enumerate().map(|(idx, button)| {
            let mut args: Vec<&str> = vec![version, mode, edition];
            let it_same: bool = button.to_lowercase() == args[idx];
            args[idx] = button;
            InlineKeyboardButton::callback(
                format!("{} {}", button, if it_same { check } else { uncheck }),
                if button == &"-" {
                    "print 😑".into()
                } else {
                    format!(
                        "option {} {} {} {} {}",
                        args[0].to_lowercase(),
                        args[1].to_lowercase(),
                        args[2].to_lowercase(),
                        match idx {
                            0 => "version",
                            1 => "mode",
                            _ => "edition",
                        },
                        button.to_lowercase()
                    )
                },
            )
        }));
    }
    keyboard
}

pub fn view_run_keyboard(
    version: &str,
    mode: &str,
    edition: &str,
    already_use_keyboard: bool,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "Run 🦀⚙️".into(),
        format!(
            "viewR {} {} {} {}",
            version, mode, edition, already_use_keyboard
        ),
    )]])
}

pub fn view_share_keyboard(
    version: &str,
    mode: &str,
    edition: &str,
    already_use_keyboard: bool,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "Share 🦀🔗".into(),
        format!(
            "viewS {} {} {} {}",
            version, mode, edition, already_use_keyboard
        ),
    )]])
}

pub fn run_keyboard(version: String, mode: String, edition: String) -> InlineKeyboardMarkup {
    option_keyboard(&version, &mode, &edition).append_row([InlineKeyboardButton::callback(
        "Run 🦀⚙️".into(),
        format!("run {} {} {}", version, mode, edition),
    )])
}

pub fn share_keyboard(version: String, mode: String, edition: String) -> InlineKeyboardMarkup {
    option_keyboard(&version, &mode, &edition).append_row([InlineKeyboardButton::callback(
        "Share 🦀🔗".into(),
        format!("share {} {} {}", version, mode, edition),
    )])
}
