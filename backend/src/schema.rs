// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        username -> Varchar,
        id -> Int4,
        passw -> Varchar,
        created_at -> Date,
    }
}

diesel::table! {
    authorized_cookies (session_id) {
        client_signature -> Nullable<Text>,
        session_id -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    authorized_cookies,
);
