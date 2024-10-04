diesel::table! {
    account (username) {
        id -> Int4,
        username -> Text,
        password -> Text,
        created_at -> Timestamp,
    }
}
