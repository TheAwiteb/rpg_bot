CREATE TABLE source_codes (
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL references users(id),
    code VARCHAR NOT NULL,
    source_code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
)