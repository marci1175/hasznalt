use anyhow::bail;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier
};
use client_type::AccountLogin;
use db_type::Account;
use diesel::{
    dsl::insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper,
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
    use std::fmt::Display;

    use diesel::{
        prelude::{Insertable, Queryable, QueryableByName},
        Selectable,
    };
    use serde::{Deserialize, Serialize};

    use crate::schema::accounts;

    #[derive(
        QueryableByName, Selectable, Queryable, Insertable, Deserialize, Serialize, Clone, Debug,
    )]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = accounts)]
    pub struct Account {
        pub username: String,
        pub passw: String,
    }

    impl Display for Account {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&serde_json::to_string(self).unwrap())
        }
    }
}

pub mod client_type {
    use serde::{Deserialize, Serialize};

    use crate::{db_type::Account, hash_password};

    #[derive(Serialize, Deserialize)]
    pub struct AccountLogin {
        pub username: String,
        pub password: String,
    }

    impl AccountLogin {
        /// This function creates an ```Accout``` from an ```AccountLogin```
        /// Please note that the password get hashed via ```Argon2```, to safely store it
        /// This function returns a result indicating the result of the hashing process
        pub fn into_server_type(&self) -> anyhow::Result<Account> {
            Ok(Account {
                username: self.username.clone(),
                passw: hash_password(self.password.clone())?,
            })
        }
    }
}

pub fn hash_password(password: String) -> anyhow::Result<String> {
    //Create argon2 hasher instance
    let argon2 = argon2::Argon2::default();

    //Create salt
    let salt = SaltString::generate(&mut OsRng);

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow::Error::msg(err.to_string()))?
        .to_string())
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
    let server_type_account = request.into_server_type()?;

    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_write()
        .run(move |conn| {
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
                        .values(&server_type_account)
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
    let argon2 = Argon2::default();

    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_only()
        .run(|conn| {
            conn.transaction(|conn| {
                let matched_account = accounts::dsl::accounts
                    //Check for username match
                    .filter(username.eq(request.username))
                    .select(Account::as_select())
                    //Check for password match
                    .load(conn)?.into_iter().find(
                        |account| {
                            argon2.verify_password(request.password.as_bytes(), &PasswordHash::new(&account.passw).unwrap()).is_ok()
                        }
                    );

                matched_account.ok_or_else(|| anyhow::Error::msg("Profile not found"))
            })
        })
        .map_err(anyhow::Error::from)
}
