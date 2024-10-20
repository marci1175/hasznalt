use axum::{
    extract::{Request, State}, middleware::{self, Next}, response::{Html, IntoResponse}, routing::{get, post}, serve, Json, Router
};
use axum_extra::extract::CookieJar;
use backend::{
    db_types::{safe_types::AccountLookup, unsafe_types::AuthorizedUser}, establish_server_state, get_account_id_account_request, get_account_login_request, get_account_register_request, get_cookie_account_request, safe_functions::{check_authenticated_account, lookup_account_from_id}, ServerState
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
        .route("/api/register", post(get_account_register_request))
        .route("/api/login", post(get_account_login_request))
        .route("/api/id_lookup", post(get_account_id_account_request))
        .route("/api/account", post(get_cookie_account_request))
        .layer(cors)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            login_persistence,
        ))
        .with_state(state);

    serve(listener, app).await?;

    Ok(())
}

async fn login_persistence(
    jar: CookieJar,
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    //Check if the user has already authenticated itself once
    if let Some(cookie_session_id) = jar.get("session_id") {
        let authorized_user = serde_json::from_str::<AuthorizedUser>(cookie_session_id.value())
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        //Validate cookie
        if let Some(authenticated_user) =
            check_authenticated_account(state.pgconnection.clone(), &authorized_user)?
        {
            dbg!(lookup_account_from_id(authenticated_user.account_id, state.pgconnection.clone())
                            .unwrap());
        }
    }

    Ok(next.run(request).await)
}
