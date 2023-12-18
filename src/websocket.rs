use serde_json::Value;
use socketioxide::extract::State;
use socketioxide::extract::{Data, SocketRef};
use tracing::info;
use crate::packets::*;
use crate::state;

pub async fn on_connect_default(socket: SocketRef) {
    info!("Client connected: {:?}", socket.id);

    socket.on("sqv:client_preflight", |socket: SocketRef, Data::<ClientPreflight>(data)| async move {
        info!("Received message: {:?}", data);
    });

    socket.on("sqv::create_session", |socket: SocketRef, Data::<Value>(data), store: State<state::MessageStore>| async move {
        info!("sqv::create_session: {:?}", data);
    });
}