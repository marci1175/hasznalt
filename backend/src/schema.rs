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
    authorized_users (session_id) {
        client_signature -> Varchar,
        session_id -> Varchar,
        account_id -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(accounts, authorized_users,);
