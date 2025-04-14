// @generated automatically by Diesel CLI.

diesel::table! {
    event_tags (tag, tag_value, event_id) {
        tag -> Text,
        tag_value -> Text,
        event_id -> Binary,
    }
}

diesel::table! {
    events (id) {
        id -> Binary,
        pubkey -> Binary,
        created_at -> BigInt,
        kind -> BigInt,
        payload -> Binary,
        deleted -> Bool,
    }
}

diesel::joinable!(event_tags -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(event_tags, events);
