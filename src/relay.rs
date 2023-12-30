use std::future;
use std::sync::Arc;
use std::time::Duration;
use futures::{FutureExt, stream, StreamExt, TryFutureExt, TryStreamExt};
use serde_json::Value;
use socketioxide::extract::{SocketRef, AckSender, Bin, State, Data};
use socketioxide::SocketIo;
use tracing::{debug, info};
use uuid::Uuid;
use crate::internal::{AckResponseExt, ClientId, create_random_namespace};
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

    socket.extensions.insert(Uuid::new_v4().to_string());

    socket.on_disconnect(|socket: SocketRef| async move {
        let client_id = socket.extensions.get::<ClientId>().unwrap().clone();
        info!("{:?} ({:?}) disconnected", socket.id, client_id);

        // todo get all clients from namespace
        //  if no clients, remove namespace
    });

    socket.on("sqw:broadcast", |socket: SocketRef, Data(data): Data<Value>, ack: AckSender, Bin(bin): Bin| async move {
        let client_id = socket.extensions.get::<ClientId>().unwrap().clone();
        info!("{:?} ({:?}) broadcasting", socket.id, client_id);

        let ack_stream = socket.broadcast()
            .timeout(Duration::from_millis(4000))
            .bin(bin)
            .emit_with_ack::<Value>("sqw:data", ClientData { id: client_id, data });

        // todo handle ack.is_err()
        let (json, binary): (Vec<Value>, Vec<Vec<Vec<u8>>>) =
            StreamExt::map(ack_stream, |ack| { ack.unwrap().transform_response() })
            .collect::<Vec<_>>().await
            .iter().cloned()
            .unzip();;

        let json = Value::Array(json);
        let binary = binary.into_iter().flatten().collect::<Vec<_>>();
        ack.bin(binary).send(json).ok();
    });

    socket.on("sqw:request", |socket: SocketRef, Data(data): Data<Value>, ack: AckSender, Bin(bin): Bin| async move {
        let client_id = socket.extensions.get::<ClientId>().unwrap().clone();
        info!("{:?} ({:?}) requesting", socket.id, client_id);

        let sockets = socket.broadcast().sockets().unwrap();
        let target = sockets.get(0);

        if let Some(target) = target {
            let responses = target
                .timeout(Duration::from_millis(4000))
                .bin(bin)
                .emit_with_ack::<Value>("sqw:data", ClientData { id: client_id, data })
                .collect::<Vec<_>>()
                .await;

            if let Ok(response) = responses.get(0).unwrap() {
                let (json, binary) = response.transform_response();
                ack.bin(binary).send(json).ok();
            }
        }
    });
}