use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    serve, Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use backend::{
    db_type::Account, establish_server_state, handle_account_login_request, handle_account_register_request, ServerState
};
use reqwest::{Method, StatusCode};
use std::path::PathBuf;
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

/// This function will register a new account depending on the request it takes.
/// It can either return ```StatusCode::CREATED```: When the account has been successfully registered
/// Or return ```StatusCode::FOUND```: When the account has been already registered, thus it will not create another one
pub async fn get_account_register_request(
    State(state): State<ServerState>,
    Json(body): Json<Account>,
) -> StatusCode {
    match handle_account_register_request(body, state) {
        Ok(_) => StatusCode::CREATED,
        Err(_err) => StatusCode::FOUND,
    }
}

pub async fn get_account_login_request(
    jar: CookieJar,
    State(state): State<ServerState>,
    Json(body): Json<Account>,
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let account = handle_account_login_request(body, state).map_err(|_| StatusCode::NOT_FOUND)?; 

    let uuid = uuid::Uuid::now_v7().to_string();

    Ok((jar.add(Cookie::new("name", "asd")), axum::Json(account.to_string())))
}
