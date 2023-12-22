use std::collections::HashSet;
use std::sync::Arc;
use futures::StreamExt;
use socketioxide::extract::{State, Data, SocketRef, AckSender, Bin};
use socketioxide::SocketIo;
use tracing::{info};
use uuid::Uuid;
use crate::data::ClientId;
use crate::packets::*;
use crate::state::NamespaceStore;

fn create_random_namespace(namespaces: &HashSet<String>) -> String {
    const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789";

    let mut s = String::new();
    let len = std::env::var("NAMESPACE_LEN")
        .unwrap_or("7".to_string())
        .parse::<usize>().unwrap();

    loop {
        s = random_string::generate(len, CHARSET);
        if !namespaces.contains(&s) {
            break;
        }
    }

    return s;
}

pub async fn on_connect_default(socket: SocketRef, io: Arc<SocketIo>) {
    info!("Client connected: {:?} on ns: {:?}", socket.id, socket.ns());

    socket.on(
        "sqw:client_preflight",
        |socket: SocketRef, Data::<ClientPreflightRequest>(data), ack: AckSender, namespace_store: State<NamespaceStore>| async move {
            info!("Client preflight: {:?}", data);

            let namespaces = namespace_store.get_all().await;
            let generated = create_random_namespace(&namespaces);
            namespace_store.insert(generated.clone()).await;

            let io_clone = io.clone();
            io.ns(format!("/{}", generated), |socket: SocketRef| {
                on_connect_dynamic(socket, io_clone)
            });

            ack.send(&ClientPreflightResponse { ns: generated, }).ok();
        }
    );
}

pub async fn on_connect_dynamic(socket: SocketRef, _io: Arc<SocketIo>) {
    info!("Client connected: {:?} on ns: {:?}", socket.id, socket.ns());

    socket.extensions.insert(ClientId(Uuid::new_v4().to_string()));

    socket.on(
        "sqw:broadcast",
        |socket: SocketRef, ack: AckSender, Bin(bin)| async move {
            info!("Client broadcast: {:?}", bin);
            let client_id = socket.extensions.get::<ClientId>().unwrap().clone();

            socket.broadcast()
                .bin(bin)
                .emit_with_ack::<ClientData>("sqw:data", ClientData { id: client_id, })
                .unwrap()
                .for_each(|_| async {
                    info!("Client broadcasted");
                });
        }
    );

    socket.on_disconnect(|socket: SocketRef| {
        info!("Client disconnected: {:?}", socket.id);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_random_namespace() {
        let vec: HashSet<String> = vec!["test1".to_string(), "test2".to_string()].into_iter().collect();
        let ns = create_random_namespace(&vec);
        println!("generated: {:?}", ns);
        assert_eq!(ns.len(), 7);
        assert!(!vec.contains(&ns));
        assert!(vec.contains(&"test2".to_string()));
    }
}