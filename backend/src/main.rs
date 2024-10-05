use std::path::PathBuf;

use anyhow::bail;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    serve, Json, Router,
};
use backend::{db_type::Account, establish_server_state, schema::account::username, ServerState};
use diesel::{
    insert_into, Connection, ExpressionMethods, OptionalExtension,
    QueryDsl, RunQueryDsl,
};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use tokio::{fs, net::TcpListener};
use tower::util::ServiceExt;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("[::]:3004").await?;

    let state = establish_server_state()?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::HEAD])
        .allow_origin(Any);

    let app = Router::new()
        //Define service
        .fallback_service(get(|req| async move {
            let res = ServeDir::new("C:\\Users\\marci\\Desktop\\hasznalt\\frontend\\dist")
                .oneshot(req)
                .await
                .unwrap();
            let status = res.status();
            match status {
                StatusCode::NOT_FOUND => {
                    let index_path =
                        PathBuf::from("C:\\Users\\marci\\Desktop\\hasznalt\\frontend\\dist")
                            .join("index.html");
                    fs::read_to_string(index_path)
                        .await
                        .map(|index_content| (StatusCode::OK, Html(index_content)).into_response())
                        .unwrap_or_else(|_| {
                            (StatusCode::INTERNAL_SERVER_ERROR, "index.html not found")
                                .into_response()
                        })
                }

                // path was found as a file in the static dir
                _ => res.into_response(),
            }
        }))
        /*
            Define api routes
        */
        //ERROR
        // .route("/*api", get(|| async { Redirect::permanent("/") }))
        .route("/api/register", post(get_account_register_request))
        .route("/api/login", post(get_account_login_request))
        .layer(cors)
        .with_state(state);

    serve(listener, app).await?;

    Ok(())
}

pub async fn get_account_register_request(
    State(state): State<ServerState>,
    Json(body): Json<Account>,
) -> String {
    match handle_account_register_request(body, state) {
        Ok(_) => String::from("200"),
        Err(_err) => String::from("400"),
    }
}

pub async fn get_account_login_request(
    State(state): State<ServerState>,
    Json(body): Json<Account>,
) -> String {
    match handle_account_login_request(body, state) {
        Ok(login) => String::from("200"),
        Err(_err) => String::from("400"),
    }
}

pub fn deserialize_into_value<'a, T: Deserialize<'a>>(
    serialized_string: &'a str,
) -> anyhow::Result<T> {
    Ok(serde_json::from_str::<T>(serialized_string)?)
}

use backend::schema::account;

/// This function is going to write data to the database and return an ```anyhow::Result<usize>```
/// If the query was unsuccessful or didnt find the user it will return ```Ok(usize)```, with the inner value being the nuber of rows inserted.
/// If the query was successfull and found the user the client requested it will return an ```Error(_)```
pub fn handle_account_register_request(request: Account, state: ServerState) -> anyhow::Result<usize> {
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
                conn
                    .transaction(|conn| insert_into(account::table).values(&request).execute(conn)).map_err(anyhow::Error::from)
            }
        })
}

/// This function is going to read data out of the database and return an ```anyhow::Result<Option<Account>>```
/// If the query was unsuccessful or didnt find the user it will return an error.
/// If the query was successfull and found the user the client requested it will return an ```Account```
pub fn handle_account_login_request(
    request: Account,
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
