use std::path::PathBuf;

use axum::{
    extract::{Path, State}, response::{Html, IntoResponse, Redirect}, routing::{get, post}, serve, Router
};
use backend::{establish_server_state, rs_type::NewAccount, ServerState};
use diesel::{insert_into, Connection, RunQueryDsl};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::{fs, net::TcpListener};
use tower_http::services::ServeDir;
use tower::util::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("[::]:3004").await?;

    let state = establish_server_state()?;

    let app = Router::new()
        //Define api routes
        .route_service("/*api", get(|| async { Redirect::permanent("/") }))
        .route("/api/register/*serde", post(get_account_request))
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
        .with_state(state);

    serve(listener, app).await?;

    Ok(())
}

pub async fn get_account_request(State(state): State<ServerState>, Path(path): Path<String>) -> String {
    match handle_account_request(path, state) {
        Ok(_) => {
            String::from("200")
        },
        Err(_err) => {
            String::from("400")
        },
    }
}

pub fn deserialize_into_value<'a, T: Deserialize<'a>>(serialized_string: &'a str) -> anyhow::Result<T> {
    return Ok(serde_json::from_str::<T>(&serialized_string)?);
}

use backend::schema::account;

pub fn handle_account_request(request: String, state: ServerState) -> anyhow::Result<()> {
    let parsed_account = deserialize_into_value::<NewAccount>(&request)?;

    state.pgconnection.lock().unwrap().build_transaction().read_write().run(|conn| {
        conn.transaction(|conn| {
            insert_into(account::table).values(&parsed_account).execute(conn)
        })
    })?;

    Ok(())
}
