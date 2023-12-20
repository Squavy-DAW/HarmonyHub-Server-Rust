use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ClientPreflightRequest {
}

#[derive(Debug, Serialize)]
pub struct ClientPreflightResponse {
    pub ns: String
}