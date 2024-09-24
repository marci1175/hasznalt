use tokio::net::TcpListener;

use axum::{extract::{ws::{WebSocket, WebSocketUpgrade}, State}, response::{Html, IntoResponse}, routing::get, serve, Router};

#[derive(Debug, Clone)]
struct AppState {}

#[tokio::main]
// #[axum::debug_handler]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    let state = AppState {

    };

    let app = Router::new()
        .route("/", get(handler))
        .with_state(state);

    serve(listener, app).await?;

    Ok(())    
}

async fn handler() -> Html<&'static str> {
    Html(include_str!("../../client/dist/index.html"))
}