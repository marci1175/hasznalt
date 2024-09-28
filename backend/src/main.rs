use std::path::PathBuf;
use reqwest::StatusCode;
use tokio::{fs, net::TcpListener};
use tower::util::ServiceExt;
use axum::{response::{Html, IntoResponse}, routing::get, serve, Router};
use tower_http::services::ServeDir;

#[derive(Debug, Clone)]
struct AppState {}

#[tokio::main]
// #[axum::debug_handler]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("[::]:3004").await?;

    let state = AppState {

    };

    let app = Router::new()
    .route("/test", get(handler))
        .fallback_service(get(|req| async move {
            let res = ServeDir::new("C:\\Users\\marci\\Desktop\\hasznalt\\frontend\\dist").oneshot(req).await.unwrap(); // serve dir is infallible
            let status = res.status();
            match status {
                // If we don't find a file corresponding to the path we serve index.html.
                // If you want to serve a 404 status code instead you can add a route check as shown in
                // https://github.com/rksm/axum-yew-setup/commit/a48abfc8a2947b226cc47cbb3001c8a68a0bb25e
                StatusCode::NOT_FOUND => {
                    let index_path = PathBuf::from("C:\\Users\\marci\\Desktop\\hasznalt\\frontend\\dist").join("index.html");
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

pub async fn handler() -> String {
    "Fasz".to_string()
}