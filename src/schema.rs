table! {
    source_codes (id) {
        id -> Integer,
        user_id -> Integer,
        code -> Text,
        source_code -> Text,
        created_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Nullable<Text>,
        telegram_id -> Text,
        telegram_fullname -> Text,
        attempts -> Integer,
        last_record -> Nullable<Timestamp>,
    }
}

joinable!(source_codes -> users (user_id));

allow_tables_to_appear_in_same_query!(source_codes, users,);
