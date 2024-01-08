use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use crate::internal::ClientId;

#[derive(Debug, Serialize)]
pub struct ClientPreflightResponse {
    pub ns: String
}

#[derive(Debug, Serialize)]
pub struct ClientData {
    pub id: ClientId,
    pub data: Value
}

#[derive(Debug, Serialize)]
pub struct ClientDisconnected {
    pub id: ClientId
}