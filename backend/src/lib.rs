use anyhow::bail;
use client_type::AccountLogin;
use db_type::Account;
use diesel::{dsl::insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use schema::account::{self, username};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

pub mod schema;

#[derive(Clone)]
pub struct ServerState {
    pub pgconnection: Arc<Mutex<PgConnection>>,
}

pub mod db_type {
    use diesel::prelude::{Insertable, Queryable, QueryableByName};
    use serde::{Deserialize, Serialize};

    use crate::schema::account;

    #[derive(QueryableByName, Queryable, Insertable, Deserialize, Serialize, Clone)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = account)]
    pub struct Account {
        pub id: i32,
        pub username: String,
        pub password: String,
        pub created_at: chrono::NaiveDateTime,
    }

    impl ToString for Account {
        fn to_string(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

}

pub mod client_type {
    use serde::{Deserialize, Serialize};

    use crate::db_type::Account;

    #[derive(Serialize, Deserialize)]
    pub struct AccountLogin {
        pub username: String,
        pub password: String,
    }

    impl AccountLogin {
        pub fn into_server_type(self) -> Account {
            Account { id: 0, username: self.username.clone(), password: self.username.clone(), created_at: chrono::NaiveDateTime::default() }
        }
    }
}

pub fn establish_server_state() -> anyhow::Result<ServerState> {
    let pgconnection = diesel::PgConnection::establish(include_str!("..\\..\\.env"))?;

    Ok(ServerState {
        pgconnection: Arc::new(Mutex::new(pgconnection)),
    })
}


pub fn deserialize_into_value<'a, T: Deserialize<'a>>(
    serialized_string: &'a str,
) -> anyhow::Result<T> {
    Ok(serde_json::from_str::<T>(serialized_string)?)
}

/// This function is going to write data to the database and return an ```anyhow::Result<usize>```
/// If the query was unsuccessful or didnt find the user it will return ```Ok(usize)```, with the inner value being the nuber of rows inserted.
/// If the query was successfull and found the user the client requested it will return an ```Error(_)```
pub fn handle_account_register_request(
    request: AccountLogin,
    state: ServerState,
) -> anyhow::Result<usize> {
    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_write()
        .run(|conn| {
            if let Ok(Some(_)) = account::dsl::account
                .filter(username.eq(&request.username))
                .first::<Account>(conn)
                .optional()
            {
                bail!("User already exists.")
            } else {
                conn.transaction(|conn| insert_into(account::table).values(&request.into_server_type()).execute(conn))
                    .map_err(anyhow::Error::from)
            }
        })
}

/// This function is going to read data out of the database and return an ```anyhow::Result<Option<Account>>```
/// If the query was unsuccessful or didnt find the user it will return an error.
/// If the query was successfull and found the user the client requested it will return an ```Account```
pub fn handle_account_login_request(
    request: AccountLogin,
    state: ServerState,
) -> anyhow::Result<Account> {
    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_only()
        .run(|conn| {
            conn.transaction(|conn| {
                account::dsl::account
                    .filter(username.eq(request.username))
                    .first::<Account>(conn)
            })
        })
        .map_err(anyhow::Error::from)
}