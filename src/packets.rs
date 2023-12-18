use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientPreflight {
    ns: String
}

#[derive(Debug, Deserialize)]
pub struct MessageIn {
    pub room: String,
    pub text: String,
}