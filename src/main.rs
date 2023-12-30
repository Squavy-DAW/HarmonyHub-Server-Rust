mod relay;
mod dotenv;
mod packets;
mod state;
mod internal;

use socketioxide::{SocketIo};
use std::env;
use std::ops::Deref;
use std::sync::Arc;
use axum::http::HeaderValue;
use socketioxide::extract::{SocketRef};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::relay::on_connect_default;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(tracing_appender::rolling::hourly(
                "./logs", "server")
                .with_max_level(tracing::Level::DEBUG)))
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout
                .with_max_level(
                    #[cfg(debug_assertions)]
                    tracing::Level::DEBUG,
                    #[cfg(not(debug_assertions))]
                    tracing::Level::INFO,
                )))
        .init();

    dotenv::load_env();

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
