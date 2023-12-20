use std::sync::Arc;
use serde_json::{to_string, Value};
use socketioxide::extract::{State, Data, SocketRef, AckSender};
use socketioxide::SocketIo;
use tracing::{info};
use crate::packets::*;
use crate::state;

pub async fn on_connect_default(socket: SocketRef, io: Arc<SocketIo>) {
    info!("Client connected: {:?} on ns: {:?}", socket.id, socket.ns());

    socket.on(
        "sqw:client_preflight",
        |data: Data<ClientPreflightRequest>, ack: AckSender| async move {
            info!("Client preflight: {:?}", data.0);

            let generated = "test".to_string();

            let io_clone = io.clone();
            io.ns(format!("/{}", generated), |socket: SocketRef| {
                on_connect_dynamic(socket, io_clone)
            });

            let res = to_string(&ClientPreflightResponse {
                ns: generated,
            }).unwrap();
            ack.send(res).unwrap();
        }
    );
}

pub async fn on_connect_dynamic(socket: SocketRef, _io: Arc<SocketIo>) {
    info!("Client connected: {:?} on ns: {:?}", socket.id, socket.ns());

    socket.on(
        "sqw::create_session",
        |socket: SocketRef, Data::<Value>(data), store: State<state::State>| async move {
        }
    );
}