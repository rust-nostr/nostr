// @generated automatically by Diesel CLI.

diesel::table! {
    event_tags (tag, tag_value, event_id) {
        #[max_length = 64]
        tag -> Varchar,
        #[max_length = 512]
        tag_value -> Varchar,
        #[max_length = 32]
        event_id -> Blob,
    }
}

diesel::table! {
    events (id) {
        #[max_length = 32]
        id -> Blob,
        #[max_length = 32]
        pubkey -> Blob,
        created_at -> Bigint,
        kind -> Bigint,
        payload -> Blob,
        deleted -> Bool,
    }
}

diesel::joinable!(event_tags -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(event_tags, events);
