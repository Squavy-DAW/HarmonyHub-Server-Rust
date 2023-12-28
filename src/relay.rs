use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use serde_json::Value;
use socketioxide::extract::{State, Data, SocketRef, AckSender, Bin};
use socketioxide::SocketIo;
use tracing::{debug, info};
use uuid::Uuid;
use crate::internal::{ClientId, create_random_namespace};
use crate::packets::*;
use crate::state::NamespaceStore;

pub async fn on_connect_default(socket: SocketRef, io: Arc<SocketIo>) {
    info!("{:?} -> default namespace", socket.id);

    socket.on(
        "sqw:client_preflight",
        |socket: SocketRef, ack: AckSender, namespace_store: State<NamespaceStore>| async move {
            let namespaces = namespace_store.get_all().await;
            let generated = create_random_namespace(&namespaces);
            namespace_store.insert(generated.clone()).await;

            let io_clone = io.clone();
            io.ns(format!("/{}", generated), |socket: SocketRef| {
                on_connect_dynamic(socket, io_clone)
            });

            info!("Created new namespace: {}", generated);

            ack.send(&ClientPreflightResponse { ns: generated, }).ok();
            socket.disconnect().ok();
        }
    );
}

pub async fn on_connect_dynamic(socket: SocketRef, io: Arc<SocketIo>) {
    info!("{:?} -> {:?}", socket.id, socket.ns());

    socket.extensions.insert(ClientId(Uuid::new_v4().to_string()));

    socket.on_disconnect(|socket: SocketRef| async move {
        info!("{:?} disconnected", socket.id);

        // todo get all clients from namespace
        //  if no clients left, remove namespace
    });

    socket.on("sqw:broadcast", |socket: SocketRef, ack: AckSender, Bin(bin): Bin| async move {
        let client_id = socket.extensions.get::<ClientId>().unwrap().clone();
        info!("{:?} broadcasting", socket.id);

        // todo get client id from response
        let (json, binary) : (Vec<Value>, Vec<Vec<Vec<u8>>>) = socket.broadcast()
            .timeout(Duration::from_millis(4000))
            .bin(bin)
            .emit_with_ack::<Value>("sqw:data", ClientData { id: client_id }).unwrap() // todo maybe add remaining json data
            .filter(|ack| futures::future::ready(ack.is_ok()))
            .map(|ack| {
                let ack = ack.unwrap();
                return (ack.data, ack.binary);
            })
            .collect::<Vec<_>>().await
            .iter().cloned()
            .unzip();

        let json = Value::Array(json);
        let binary = binary.into_iter().flatten().collect::<Vec<_>>();
        ack.bin(binary).send(json).ok();
    });

    socket.on("sqw:request", |socket: SocketRef, ack: AckSender, Bin(bin): Bin| async move {
        let client_id = socket.extensions.get::<ClientId>().unwrap().clone();

        let sockets = socket.broadcast().sockets().unwrap();
        let target = sockets.get(0);

        if let Some(target) = target {
            info!("{:?} requesting to {:?}", socket.id, target.id);
            let responses = target
                .timeout(Duration::from_millis(4000))
                .bin(bin)
                .emit_with_ack::<Value>("sqw:data", ClientData { id: client_id }).unwrap() // todo maybe add remaining json data
                .collect::<Vec<_>>().await;

            let response = responses
                .get(0).unwrap();

            if let Ok(response) = response {
                let json = response.data.clone();
                let binary = response.binary.clone();
                ack.bin(binary).send(json).ok();
            }
        }
    });
}