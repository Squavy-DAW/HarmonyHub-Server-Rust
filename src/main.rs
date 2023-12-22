mod relay;
mod dotenv;
mod packets;
mod state;
mod data;

// use websocket::on_default_connect;
use socketioxide::{SocketIo};
use std::env;
use std::ops::Deref;
use std::sync::Arc;
use axum::http::HeaderValue;
use serde_json::Value;
use socketioxide::extract::{Bin, Data, SocketRef};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use crate::relay::on_connect_default;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::load_env();

    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (socket_io_layer, io) = SocketIo::builder()
        .with_state(state::NamespaceStore::default())
        .build_layer();

    let io_clone = Arc::new(io.clone());
    io.ns("/", move |socket: SocketRef| {
        return on_connect_default(socket, io_clone);
    });

    let app = axum::Router::new()
        .nest_service("/", ServeDir::new("dashboard"))
        .layer(ServiceBuilder::new()
            .layer(CorsLayer::new()
                .allow_origin(env::var("CLIENT_ORIGIN").expect("CLIENT_ORIGIN must be set")
                    .parse::<HeaderValue>()
                    .unwrap()))
            .layer(socket_io_layer));

    let server_port = env::var("SERVER_PORT").expect("SERVER_PORT must be set");

    info!("Starting server on port {}...", server_port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", server_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
