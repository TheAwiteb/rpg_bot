use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

struct Button {
    text: String,
    b_type: String,
    content: String,
}

impl Button {
    fn new(text: &str, b_type: &str, content: &str) -> Button {
        Button {
            text: text.into(),
            b_type: b_type.into(),
            content: content.into(),
        }
    }
}

fn keyboard_maker(buttons: Vec<Vec<Button>>) -> InlineKeyboardMarkup {
    let mut keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::default();
    for (idx, row) in buttons.into_iter().enumerate() {
        for button in row {
            keyboard = keyboard.append_to_row(
                idx,
                if button.b_type == String::from("url") {
                    InlineKeyboardButton::url(button.text, button.content)
                } else {
                    InlineKeyboardButton::callback(button.text, button.content)
                },
            );
        }
    }
    keyboard
}

pub fn repo_keyboard() -> InlineKeyboardMarkup {
    keyboard_maker(vec![vec![Button::new(
        "Repository 🦀",
        "url",
        "https://github.com/TheAwiteb/rpg_bot",
    )]])
}
