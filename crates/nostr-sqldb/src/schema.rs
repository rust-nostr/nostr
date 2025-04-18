// @generated automatically by Diesel CLI.

diesel::table! {
    event_tags (tag, tag_value, event_id) {
        tag -> Text,
        tag_value -> Text,
        #[max_length = 64]
        event_id -> Varchar,
    }
}

diesel::table! {
    events (id) {
        #[max_length = 64]
        id -> Varchar,
        #[max_length = 64]
        pubkey -> Varchar,
        created_at -> Int8,
        kind -> Int8,
        payload -> Bytea,
        #[max_length = 128]
        signature -> Varchar,
        deleted -> Bool,
    }
}

diesel::joinable!(event_tags -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(
    event_tags,
    events,
);
