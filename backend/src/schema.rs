// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        username -> Varchar,
        id -> Int4,
        passw -> Varchar,
        created_at -> Date,
    }
}
