use anyhow::{bail, Error};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Request, State}, http::HeaderMap, middleware::Next, response::{IntoResponse, Redirect}, Json
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use db_types::{
    safe_types::AccountLookup,
    unsafe_types::{self, Account, AuthorizedUser},
};
use diesel::{
    dsl::insert_into, r2d2::ConnectionManager, Connection, ExpressionMethods, OptionalExtension,
    PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use reqwest::StatusCode;
use safe_functions::{
    check_authenticated_account, handle_account_login_request, handle_account_register_request,
    lookup_account_from_id, record_authenticated_account,
};
use schema::{
    accounts::{self, username},
    authorized_users::{self, session_id},
};
use sha2::Sha256;
use std::collections::BTreeMap;

pub mod schema;

pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct ServerState {
    pub pgconnection: PgPool,
}

pub mod db_types {
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

    pub mod unsafe_types {
        use crate::db_types::*;

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
        pub struct AccountLookup {
            /// The username of the requested user
            pub username: String,
            /// The UUID of the requested user
            pub id: i32,
            /// The `Argon2` hashed password of the requested user
            pub passw: String,
            /// The timestamp taken when the account was created
            pub created_at: chrono::NaiveDate,
        }

        impl Display for AccountLookup {
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
            pub fn from_account(account: &AccountLookup, client_sig: String) -> Self {
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
    }

    pub mod safe_types {
        use crate::db_types::*;

        #[derive(Queryable, Selectable, QueryableByName, Serialize, Deserialize, Clone, Debug)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        #[diesel(table_name = accounts)]
        pub struct AccountLookup {
            /// The username of the requested user
            pub username: String,
            /// The UUID of the requested user
            pub id: i32,
            /// The timestamp taken when the account was created
            pub created_at: chrono::NaiveDate,
        }

        impl Display for AccountLookup {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&serde_json::to_string(self).unwrap())
            }
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
    let database_url = include_str!("..\\..\\.env");

    let connection_manager = ConnectionManager::new(database_url);

    let pool = r2d2::Builder::new().build(connection_manager)?;

    Ok(ServerState { pgconnection: pool })
}

/// This mod contains `unsafe` function which **will** reveal sensitive information.
/// These functions should **ONLY** be used in the backend where data security is verified and guaranteed.
/// The `safe_functions` and `unsafe_functions` mod contains functions which make queries to the database.
pub mod unsafe_functions {
    use crate::*;

    /// This function looks up the full account information and returns an ```unsafe_types::AccountLookup``` instance.
    /// Please note that this function should **NEVER** be used to return data to anywhere other then backend.
    pub fn __lookup_account_from_id_unsafe(
        id: i32,
        pgconnection: PgPool,
    ) -> anyhow::Result<unsafe_types::AccountLookup> {
        pgconnection
            .get()?
            .build_transaction()
            .read_only()
            .run(move |conn| {
                conn.transaction(|conn| {
                    let matched_account: Option<unsafe_types::AccountLookup> =
                        accounts::dsl::accounts
                            .filter(accounts::dsl::id.eq(id))
                            .first::<unsafe_types::AccountLookup>(conn)
                            .ok();

                    matched_account.ok_or_else(|| anyhow::Error::msg("Profile not found"))
                })
            })
            .map_err(anyhow::Error::from)
    }
}

/// This mod of functions contain `safe` functions which can not reveal sensitive information.
/// When creating a function in this mod please verify the intergrity.
/// The `safe_functions` and `unsafe_functions` mod contains functions which make queries to the database.
pub mod safe_functions {
    use crate::*;
    /// This function looks up the public information of an account based on their UUID.
    /// This function will return an error if the user doesnt exist.
    pub fn lookup_account_from_id(id: i32, pgconnection: PgPool) -> anyhow::Result<AccountLookup> {
        pgconnection
            .get()?
            .build_transaction()
            .read_only()
            .run(move |conn| {
                conn.transaction(|conn| {
                    let matched_account: Option<AccountLookup> = accounts::dsl::accounts
                        .filter(accounts::dsl::id.eq(id))
                        .select(AccountLookup::as_select())
                        .first(conn)
                        .ok();

                    matched_account.ok_or_else(|| anyhow::Error::msg("Profile not found"))
                })
            })
            .map_err(anyhow::Error::from)
    }

    /// This function is going to write data to the database and return an ```anyhow::Result<usize>```
    /// If the query was unsuccessful or didnt find the user it will return ```Ok(usize)```, with the inner value being the nuber of rows inserted.
    /// If the query was successful and found the user the client requested it will return an ```Error(_)```
    pub fn handle_account_register_request(
        request: Account,
        pgconnection: PgPool,
        client_headers: HeaderMap,
    ) -> anyhow::Result<usize> {
        pgconnection
            .get()?
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
        pgconnection: PgPool,
    ) -> anyhow::Result<unsafe_types::AccountLookup> {
        let argon2 = Argon2::default();

        pgconnection
            .get()?
            .build_transaction()
            .read_only()
            .run(|conn| {
                conn.transaction(|conn| {
                    let matched_account: Option<unsafe_types::AccountLookup> =
                        accounts::dsl::accounts
                            //Check for username match
                            .filter(username.eq(request.username))
                            .select(unsafe_types::AccountLookup::as_select())
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
        pgconnection: PgPool,
    ) -> anyhow::Result<usize> {
        pgconnection
            .get()?
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
        pgconnection: PgPool,
        authorized_user: &AuthorizedUser,
    ) -> Result<Option<AuthorizedUser>, StatusCode> {
        pgconnection
            .get()
            .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
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
                                && auth_user.account_id == auth_user.account_id && auth_user.client_signature == authorized_user.client_signature
                        });

                    Ok(matched_authorized_account)
                })
            })
            .map_err(|_: Error| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// This function will register a new account depending on the request it takes.
/// It can either return ```StatusCode::CREATED```: When the account has been successfuly registered
/// Or return ```StatusCode::FOUND```: When the account has been already registered, thus it will not create another one
pub async fn get_account_register_request(
    State(state): State<ServerState>,
    header: HeaderMap,
    Json(body): Json<Account>,
) -> StatusCode {
    match handle_account_register_request(body, state.pgconnection.clone(), header) {
        Ok(_) => StatusCode::CREATED,
        Err(_err) => StatusCode::FOUND,
    }
}

/// This function will create a request to the database whether the account's username is found.
/// If the password to that account matches it will create an authenticated session_id and set the client's storage
/// If the account is either not found or an invalid password is entered this function will return ```StatusCode::NOT_FOUND```
pub async fn get_account_login_request(
    jar: CookieJar,
    State(state): State<ServerState>,
    header: HeaderMap,
    Json(body): Json<Account>,
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let account = handle_account_login_request(body, state.pgconnection.clone())
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let authorized_user = AuthorizedUser::from_account(
        &account,
        sha256::digest(
            header
                .values()
                .map(|val| val.clone().as_bytes().to_vec())
                .collect::<Vec<Vec<u8>>>()
                .concat(),
        ),
    );

    // If there is an existing record with the same session id, but different client signature it means that the client may have changed host computer or the session id got stolen.
    if check_authenticated_account(state.clone().pgconnection, &authorized_user)?.is_none() {
        //Create a record if there wasnt a vail session id already
        record_authenticated_account(&authorized_user, state.pgconnection.clone())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok((
        jar.add(
            Cookie::build(Cookie::new("session_id", authorized_user.to_string()))
                .permanent()
                .path("/")
                .http_only(false)
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .build(),
        ),
        axum::Json(account.to_string()),
    ))
}

/// This function will create a request to the database to find the account specified in the ID argument
/// If the account is found this function  will return a ```Json<safe_types::AccountLookup>```
/// If the account is not found it wil return ```StatusCode::NOT_FOUND```
pub async fn get_account_id_account_request(
    State(state): State<ServerState>,
    Json(id): Json<i32>,
) -> Result<Json<AccountLookup>, StatusCode> {
    let account = lookup_account_from_id(id, state.pgconnection.clone())
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(account))
}

/// This function takes a ```Json<AuthorizedUser>``` and validates it with the database, then it looks up the account based on the account's id then returns the ```AccountLookup``` instance from the database.
/// If the account based on that id is not found it will return ```StatusCode::NOT_FOUND```  
/// If the ```AuthorizedUser``` instance is invalid it will return ```StatusCode::BAD_REQUEST```  
pub async fn get_cookie_account_request(
    State(state): State<ServerState>,
    jar: CookieJar,
) -> Result<Json<AccountLookup>, (CookieJar, StatusCode)> {
    if let Some(session_id_value) = jar.get("session_id") {
        let authorized_user = serde_json::from_str::<AuthorizedUser>(session_id_value.value()).map_err(|_| (jar.clone().remove("session_id"), StatusCode::BAD_REQUEST))?;

        //Check for the user's ```AuthorizedUser``` instance, if found return the ```AccountLookup``` instance of the account from the database.
        if let Ok(Some(authenticated_user)) =
            check_authenticated_account(state.pgconnection.clone(), &authorized_user)
        {
            Ok(Json(lookup_account_from_id(authenticated_user.account_id, state.pgconnection.clone()).map_err(|_| (jar.remove("session_id"), StatusCode::NOT_FOUND))?))
        }
        else {
            Err((jar.remove("session_id"), StatusCode::BAD_REQUEST))
        }
    }
    else {
        Err((jar.remove("session_id"), StatusCode::CONTINUE))
    }
}

pub fn get_claims_from_str(
    encrypted_string: &str,
    secret: &[u8],
) -> anyhow::Result<BTreeMap<String, String>> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret)?;

    let claims: BTreeMap<String, String> = encrypted_string.verify_with_key(&key)?;

    Ok(claims)
}

pub fn create_claims(cookies: BTreeMap<String, String>, secret: &[u8]) -> anyhow::Result<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret)?;

    let token_string = cookies.sign_with_key(&key)?;

    Ok(token_string)
}

pub async fn account_redirecting(
    jar: CookieJar,
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    //Check if the user has already authenticated itself once
    if let Some(cookie_session_id) = jar.get("session_id") {
        //Get path URI
        let request_path = request.uri();

        //Check if the user has entered forbidden path
        if request_path == "/login" || request_path == "/register" {
            let authorized_user = serde_json::from_str::<AuthorizedUser>(cookie_session_id.value())
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
            //Validate cookie we will just redirect if valid
            if (check_authenticated_account(state.pgconnection.clone(), &authorized_user)?).is_some()
            {
                //If we have a valid cookie we automaticly redirect to the home page
                return Ok(Redirect::to("/").into_response());
            }
        }
    }

    Ok(next.run(request).await)
}