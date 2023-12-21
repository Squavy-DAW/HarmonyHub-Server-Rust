mod webserver;
mod websocket;
mod dotenv;
mod packets;
mod state;

// use websocket::on_default_connect;
use socketioxide::{SocketIo};
use std::env;
use std::sync::Arc;
use axum::http::HeaderValue;
use socketioxide::extract::SocketRef;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use webserver::RouterExt;
use crate::websocket::on_connect_default;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::load_env();

    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let socket_store = state::State::default();
    let (socket_io_layer, io) = SocketIo::builder()
        .with_state(socket_store)
        .build_layer();

    let io_clone = Arc::new(io.clone());
    io.ns("/", move |socket: SocketRef| {
        return on_connect_default(socket, io_clone);
    });

    let app = axum::Router::new()
        .configure_webserver()
        .layer(CorsLayer::new()
            .allow_origin(env::var("CLIENT_ORIGIN").expect("CLIENT_ORIGIN must be set")
                .parse::<HeaderValue>()
                .unwrap()
            ))
        .layer(socket_io_layer);

    let server_port = env::var("SERVER_PORT").expect("SERVER_PORT must be set");

    info!("Starting server on port {}...", server_port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", server_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
