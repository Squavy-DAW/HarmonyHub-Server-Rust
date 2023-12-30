use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use serde_json::Value;
use socketioxide::extract::{SocketRef, AckSender, Bin, State, Data};
use socketioxide::SocketIo;
use tracing::info;
use uuid::Uuid;
use crate::internal::{AckResponseExt, ClientId, create_random_namespace};
use crate::packets::*;
use crate::state::NamespaceStore;

/// This route gets called when a client initially connects to the server.
pub async fn on_connect_default(socket: SocketRef, io: Arc<SocketIo>) {
    info!("{:?} -> default namespace", socket.id);

    /// Is called when the client wants to create a new session by preflighting the request.
    /// The server then responds with a fresh namespace that the client can connect to.
    /// Immediately after, the client is being disconnected from this route.
    ///
    /// The server creates the namespace by calling `io.ns(...)`.
    /// Simultaneously, the namespace is also added to a list of active namespaces.
    /// _Keep this list as long as we don't have access to socketioxide's own internal Hashmap router._
    socket.on("sqw:client_preflight", |socket: SocketRef, ack: AckSender, namespace_store: State<NamespaceStore>| async move {
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
    });
}

/// This route gets called when a client connects to a session
/// The client must provide the session code in the URL and is then automatically added to the session
/// Currently, there is no authentication and clients are not restricted.
pub async fn on_connect_dynamic(socket: SocketRef, io: Arc<SocketIo>) {
    info!("{:?} -> {:?}", socket.id, socket.ns());

    socket.extensions.insert(Uuid::new_v4().to_string());

    /// Called, when a client intentionally disconnects or looses connection.
    /// Todo: Checks if all clients of that namespace are disconnected and free the session again.
    socket.on_disconnect(|socket: SocketRef| async move {
        let client_id = socket.extensions.get::<ClientId>().unwrap().clone();
        info!("{:?} ({:?}) disconnected", socket.id, client_id);
    });

    /// One client broadcasts an event to all other clients in this namespace.
    /// The server accepts the request, sends it in a different format to all clients
    /// and awaits a response from each client. These responses are then packed together
    /// in a list and sent as one big chunk back to the broadcaster.
    ///
    /// The incoming format is of this structure:
    /// ```js
    /// {some_data:"hello world!"}, "nice", ArrayBuffer[0,30,2,53,81,2,10], ArrayBuffer[2,5,4]
    /// ```
    /// Note that the first half of the data arguments is JSON serialized data
    /// and the other half is always one (or more) arraybuffers.
    /// This restriction is caused by the parser of socketioxide.
    ///
    /// The broadcasted packet looks like this:
    /// It contains information about the broadcaster (broadcaster-id), the raw JSON data,
    /// and a list of arraybuffers.
    /// ```js
    /// {id: <broadcaster-id>, data: [{some_data:"hello world!"}, "nice"]}, [ArrayBuffer[0,30,2,53,81,2,10], ArrayBuffer[2,5,4]]
    /// ```
    ///
    /// The response of a client can be represented just like the initial broadcast data.
    /// The transformation of the socketioxide packet to the final data structure happens in `AckResponseExt::transform_response()`
    /// It transforms the individual data to tuples so that they can be used to assemble the final response.
    /// ```js
    /// {some_other_data:"cool!"}, ArrayBuffer[5,32,36,4,8], ArrayBuffer[2,5,4]
    ///
    ///     ⬇︎
    ///
    /// {
    ///     id: <socket-id>
    ///     json: {some_other_data:"cool!"}                             // raw JSON
    ///     binary: [ArrayBuffer[5,32,36,4,8], ArrayBuffer[2,5,4]]      // list of binary data
    /// }
    /// ```
    ///
    /// These results are then packed together into a vec, flattened out and finally sent back to the
    /// initial broadcaster. This may cause problems with different array lengths of different client's responses,
    /// as there would be no way to know which client sent how many arraybuffers. Currently, all clients
    /// NEED to send back exactly the same count of arraybuffers in order to distinguish between them.
    /// The data looks something like this where the response comes from two clients:
    /// ```js
    /// [{
    ///     id: <client-1-id>
    ///     data: { some_other_data: "cool!" }
    /// }, {
    ///     id: <client-2-id>
    ///     data: "very nice"
    /// }, [ArrayBuffer[5,32,36,4,8],               // this belongs to client 1
    ///     ArrayBuffer[2,5,4],                     // -,,-
    ///     ArrayBuffer[48,13,49,4,18],             // this belongs to client 2
    ///     ArrayBuffer[39,1,3,23,10,29,3]]]        // -,,-
    /// ```
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
            .unzip();

        let json = Value::Array(json);
        let binary = binary.into_iter().flatten().collect::<Vec<_>>();
        ack.bin(binary).send(json).ok();
    });

    /// One client requests data from another client. This target is in this case currently the first client
    /// in the (unsorted) socket list, which can be basically any client in this namespace except the requester themselves.
    /// When the request is sent, a response is awaited, which will be forwarded to the requester once arrived.
    ///
    /// The incoming format is of this structure:
    /// ```js
    /// {some_data:"hello world!"}, "nice", ArrayBuffer[0,30,2,53,81,2,10], ArrayBuffer[2,5,4]
    /// ```
    /// Note that the first half of the data arguments is JSON serialized data
    /// and the other half is always one (or more) arraybuffers.
    /// This restriction is caused by the parser of socketioxide.
    ///
    /// The request packet looks like this:
    /// It contains information about the requester (requester-id), the raw JSON data,
    /// and a list of arraybuffers.
    /// ```js
    /// {id: <requester-id>, data: [{some_data:"hello world!"}, "nice"]}, [ArrayBuffer[0,30,2,53,81,2,10], ArrayBuffer[2,5,4]]
    /// ```
    /// The response of the client can be represented just like the initial request data.
    /// The transformation of the socketioxide packet to the final data structure happens in `AckResponseExt::transform_response()`
    /// It transforms the individual data to tuples so that they can be used to send back the final response.
    /// ```js
    /// {some_other_data:"cool!"}, ArrayBuffer[5,32,36,4,8], ArrayBuffer[2,5,4]
    ///
    ///     ⬇︎
    ///
    /// {
    ///     id: <socket-id>
    ///     json: {some_other_data:"cool!"}                             // raw JSON
    ///     binary: [ArrayBuffer[5,32,36,4,8], ArrayBuffer[2,5,4]]      // list of binary data
    /// }
    /// ```
    /// This data is finally sent back to the initial requester exactly as received from the client, except with an
    /// additional client-id so that the requester knows from whom the data came.
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