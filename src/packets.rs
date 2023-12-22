use serde::{Deserialize, Serialize};
use crate::data::ClientId;

#[derive(Debug, Deserialize)]
pub struct ClientPreflightRequest {
}

#[derive(Debug, Serialize)]
pub struct ClientPreflightResponse {
    pub ns: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientData {
    pub id: ClientId,
}