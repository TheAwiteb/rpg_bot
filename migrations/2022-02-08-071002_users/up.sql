CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY,
    username VARCHAR,
    telegram_id VARCHAR NOT NULL,
    telegram_fullname VARCHAR NOT NULL,
    language VARCHAR NOT NULL DEFAULT "English 🇺🇸",
    attempts INTEGER NOT NULL DEFAULT 0,
    attempts_maximum INTEGER NOT NULL DEFAULT 100,
    last_command_record TIMESTAMP,
    last_button_record TIMESTAMP
)