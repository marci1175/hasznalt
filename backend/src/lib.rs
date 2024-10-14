use anyhow::{bail, Error};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use db_type::{Account, AccountLookupSafe, AuthorizedUser, __AccountLookupUnsafe};
use diesel::{
    dsl::insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use reqwest::StatusCode;
use schema::{
    accounts::{self, username},
    authorized_users::{self, account_id, session_id},
};
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

    use crate::{
        hash_password,
        schema::{
            accounts,
            authorized_users::{self},
        },
    };

    #[derive(
        QueryableByName, Selectable, Queryable, Insertable, Deserialize, Serialize, Clone, Debug,
    )]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = accounts)]
    /// This struct is used when there are incoming requests from clients.
    /// This is a common message for clients to request logging in.
    pub struct Account {
        /// The username of the account the user wants to log in
        pub username: String,
        /// The password of the account the user wants to log in
        /// This field contains the password in plaintext as it is only hashed on the serverside to prevent MITM attacks.
        pub passw: String,
    }

    impl Account {
        /// This fucntion prepares this ```Account``` instance to be stored in a database.
        /// Please note that the password get hashed via ```Argon2```
        /// This function returns a result indicating the result of the hashing process
        pub fn into_storable(&self) -> Account {
            Account {
                username: self.username.clone(),
                passw: hash_password(&self.passw).unwrap(),
            }
        }
    }

    impl Display for Account {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&serde_json::to_string(self).unwrap())
        }
    }

    #[derive(Queryable, Selectable, QueryableByName, Serialize, Deserialize, Clone, Debug)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = accounts)]
    /// This struct is used when returning ```Account``` instances from the database.
    /// This struct contains fields which `PostgreSQL` would fill out automatically.
    /// This should **NEVER** be used anywhere else than backend.
    pub struct __AccountLookupUnsafe {
        /// The username of the requested user
        pub username: String,
        /// The UUID of the requested user
        pub id: i32,
        /// The `Argon2` hashed password of the requested user
        pub passw: String,
        /// The timestamp taken when the account was created
        pub created_at: chrono::NaiveDate,
    }

    impl Display for __AccountLookupUnsafe {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&serde_json::to_string(self).unwrap())
        }
    }

    #[derive(
        QueryableByName,
        Selectable,
        Queryable,
        Insertable,
        Deserialize,
        Serialize,
        Clone,
        Debug,
        Default,
    )]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = authorized_users)]
    /// This struct is used when returning a cookie to the user who has logged in.
    /// This struct is stored as a cookie, and is a way of maintaining the logged in state.
    /// This cookie should never be shared
    pub struct AuthorizedUser {
        /// 0th Authentication for the logged in user / owner of the cookie
        pub client_signature: String,
        /// Session id of the cookie owner, the account paired to this session id can be looked up in the database
        pub session_id: String,
        /// The UUID of the account which this session id is linked to
        pub account_id: i32,
    }

    impl AuthorizedUser {
        /// This function takes an ```Account``` and a ```client_sig``` and turns it into a ```Deserializeable``` ```AuthorizedUser``` instance.
        /// This instance can be used to be store as a cookie.
        pub fn from_account(account: &__AccountLookupUnsafe, client_sig: String) -> Self {
            let session_id = uuid::Uuid::now_v7().to_string();

            Self {
                client_signature: client_sig,
                session_id,
                account_id: account.id,
            }
        }
    }

    impl Display for AuthorizedUser {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&serde_json::to_string(self).unwrap())
        }
    }

    #[derive(Queryable, Selectable, QueryableByName, Serialize, Deserialize, Clone, Debug)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = accounts)]
    pub struct AccountLookupSafe {
        /// The username of the requested user
        pub username: String,
        /// The UUID of the requested user
        pub id: i32,
        /// The timestamp taken when the account was created
        pub created_at: chrono::NaiveDate,
    }

    impl Display for AccountLookupSafe {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&serde_json::to_string(self).unwrap())
        }
    }
}

/// This function takes a password argument which it hashes with ```Argon2``` via the default hasher settings.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    //Create argon2 hasher instance
    let argon2 = argon2::Argon2::default();

    //Create salt
    let salt = SaltString::generate(&mut OsRng);

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow::Error::msg(err.to_string()))?
        .to_string())
}

/// This function establishes the ```ServerState``` instance
pub fn establish_server_state() -> anyhow::Result<ServerState> {
    // Connenct to Database
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
/// If the query was successful and found the user the client requested it will return an ```Error(_)```
pub fn handle_account_register_request(
    request: Account,
    state: ServerState,
) -> anyhow::Result<usize> {
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
                        .values(&request.into_storable())
                        .execute(conn)
                })
                .map_err(anyhow::Error::from)
            }
        })
}

/// This function is going to read data out of the database and return an ```anyhow::Result<Option<Account>>```
/// If the query was unsuccessful or didnt find the user it will return an error.
/// If the query was successful and found the user the client requested it will return an ```Account```
pub fn handle_account_login_request(
    request: Account,
    state: ServerState,
) -> anyhow::Result<__AccountLookupUnsafe> {
    let argon2 = Argon2::default();

    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_only()
        .run(|conn| {
            conn.transaction(|conn| {
                let matched_account: Option<__AccountLookupUnsafe> = accounts::dsl::accounts
                    //Check for username match
                    .filter(username.eq(request.username))
                    .select(__AccountLookupUnsafe::as_select())
                    //Check for password match
                    .load(conn)?
                    .into_iter()
                    .find(|account_lookup| {
                        argon2
                            .verify_password(
                                request.passw.as_bytes(),
                                &PasswordHash::new(&account_lookup.passw).unwrap(),
                            )
                            .is_ok()
                    });

                matched_account.ok_or_else(|| anyhow::Error::msg("Profile not found"))
            })
        })
        .map_err(anyhow::Error::from)
}

/// This function takes an ```&AuthorizedUser``` instance which it writes to the database, so that it can be accessed later to authenticate the user
pub fn record_authenticated_account(
    authorized_user: &AuthorizedUser,
    state: ServerState,
) -> anyhow::Result<usize> {
    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_write()
        .run(move |conn| {
            conn.transaction(|conn| {
                insert_into(authorized_users::table)
                    .values(authorized_user)
                    .execute(conn)
            })
            .map_err(anyhow::Error::from)
        })
        .map_err(anyhow::Error::from)
}

/// This function takes an ```&AuthorizedUser``` instance which it check for in the database, so it authenticate the user
pub fn check_authenticated_account(
    pgconnection: Arc<std::sync::Mutex<PgConnection>>,
    authorized_user: &AuthorizedUser,
) -> Result<Option<AuthorizedUser>, StatusCode> {
    pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_only()
        .run(|conn| {
            conn.transaction(|conn| {
                let matched_authorized_account = authorized_users::dsl::authorized_users
                    .filter(session_id.eq(authorized_user.session_id.clone()))
                    .select(AuthorizedUser::as_select())
                    .load(conn)?
                    .into_iter()
                    .find(|auth_user| {
                        auth_user.client_signature == authorized_user.client_signature
                            && auth_user.account_id == auth_user.account_id
                    });

                Ok(matched_authorized_account)
            })
        })
        .map_err(|_: Error| StatusCode::INTERNAL_SERVER_ERROR)
}

/// This function looks up the full account information and returns an ```__AccountLookupUnsafe``` instance.
/// Please note that this function should **NEVER** be used to return data to anywhere other then backend.
pub fn __lookup_account_from_id_unsafe(
    id: i32,
    state: ServerState,
) -> anyhow::Result<__AccountLookupUnsafe> {
    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_only()
        .run(move |conn| {
            conn.transaction(|conn| {
                let matched_account: Option<__AccountLookupUnsafe> = accounts::dsl::accounts
                    .filter(accounts::dsl::id.eq(id))
                    .first::<__AccountLookupUnsafe>(conn)
                    .ok();

                matched_account.ok_or_else(|| anyhow::Error::msg("Profile not found"))
            })
        })
        .map_err(anyhow::Error::from)
}

/// This function looks up the public information of an account based on their UUID.
/// This function will return an error if the user doesnt exist.
pub fn lookup_account_from_id(id: i32, state: ServerState) -> anyhow::Result<AccountLookupSafe> {
    state
        .pgconnection
        .lock()
        .unwrap()
        .build_transaction()
        .read_only()
        .run(move |conn| {
            conn.transaction(|conn| {
                let matched_account: Option<AccountLookupSafe> = accounts::dsl::accounts
                    .filter(accounts::dsl::id.eq(id))
                    .select(AccountLookupSafe::as_select())
                    .first(conn)
                    .ok();

                matched_account.ok_or_else(|| anyhow::Error::msg("Profile not found"))
            })
        })
        .map_err(anyhow::Error::from)
}
