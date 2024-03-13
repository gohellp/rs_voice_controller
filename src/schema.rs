// @generated automatically by Diesel CLI.

diesel::table! {
    voices_info (id) {
        id -> Integer,
        channel_id -> Text,
        owner_id -> Text,
    }
}
