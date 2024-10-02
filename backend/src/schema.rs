diesel::table! {
    account (id) {
        id -> Int4,
        username -> Text,
        password -> Text,
        created_at -> Timestamp,
    }
}
