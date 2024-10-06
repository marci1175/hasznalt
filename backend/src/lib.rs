use anyhow::bail;
use client_type::AccountLogin;
use db_type::Account;
use diesel::{
    dsl::{insert_into, Select},
    Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use schema::accounts::{self, username};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

pub mod schema;

#[derive(Clone)]
pub struct ServerState {
    pub pgconnection: Arc<Mutex<PgConnection>>,
}

pub mod db_type {
    use diesel::{
        prelude::{Insertable, Queryable, QueryableByName},
        Selectable,
    };
    use serde::{Deserialize, Serialize};

    use crate::schema::accounts;

    #[derive(QueryableByName, Selectable, Queryable, Insertable, Deserialize, Serialize, Clone, Debug)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = accounts)]
    pub struct Account {
        pub username: String,
        pub passw: String,
        pub created_at: chrono::NaiveDate,
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
            Account {
                username: self.username.clone(),
                passw: self.username.clone(),
                created_at: chrono::NaiveDate::default(),
            }
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
            if let Ok(Some(_)) = accounts::dsl::accounts
                .filter(username.eq(&request.username))
                .select(Account::as_select())
                .first::<Account>(conn)
                .optional()
            {
                bail!("User already exists.")
            } else {
                conn.transaction(|conn| {
                    insert_into(accounts::table)
                        .values(&request.into_server_type())
                        .execute(conn)
                })
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
                accounts::dsl::accounts
                    .filter(username.eq(request.username))
                    .select(Account::as_select())
                    .first::<Account>(conn)
            })
        })
        .map_err(anyhow::Error::from)
}
