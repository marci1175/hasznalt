use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::{get, post},
    serve, Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use backend::{
    db_types::{safe_types::AccountLookup, unsafe_types::{Account, AuthorizedUser}},
    establish_server_state,
    safe_functions::{
        check_authenticated_account, handle_account_login_request, handle_account_register_request,
        lookup_account_from_id, record_authenticated_account,
    },
    ServerState,
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
        .route("/api/account", post(get_account_account_request))
        .layer(cors)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            login_persistence,
        ))
        .with_state(state);

    serve(listener, app).await?;

    Ok(())
}

/// This function will register a new account depending on the request it takes.
/// It can either return ```StatusCode::CREATED```: When the account has been successfuly registered
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
    let account =
        handle_account_login_request(body, state.clone()).map_err(|_| StatusCode::NOT_FOUND)?;

    let authorized_user = AuthorizedUser::from_account(&account, String::new());

    record_authenticated_account(&authorized_user, state.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

async fn login_persistence(
    jar: CookieJar,
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    //Check if the user has already authenticated itself once
    if let Some(session_id) = jar.get("session_id") {
        let authorized_user = serde_json::from_str::<AuthorizedUser>(session_id.value())
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        //Validate cookie
        if let Some(authenticated_user) =
            check_authenticated_account(state.pgconnection.clone(), &authorized_user)?
        {
            lookup_account_from_id(authenticated_user.account_id, state.clone()).unwrap();
        }
    }

    Ok(next.run(request).await)
}

pub async fn get_account_account_request(
    State(state): State<ServerState>,
    Json(id): Json<i32>,
) -> Result<Json<AccountLookup>, StatusCode> {
    let account = lookup_account_from_id(id, state).map_err(|_| StatusCode::NOT_FOUND)?;
    
    //Fix double converting to json
    Ok(Json(account))
}
