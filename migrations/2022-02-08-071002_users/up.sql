CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY,
    username VARCHAR,
    telegram_id VARCHAR NOT NULL,
    telegram_fullname VARCHAR NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_record TIMESTAMP
)
