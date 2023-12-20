use std::collections::{HashMap, VecDeque};
use socketioxide::adapter::Room;
use socketioxide::extract::SocketRef;
use tokio::sync::RwLock;

pub type SessionStore = HashMap<Room, VecDeque<(SocketRef, String)>>; // session -> [socket, user_id]

#[derive(Default)]
pub struct State {
    pub sessions: RwLock<SessionStore>
}

impl State {
    pub async fn insert(&self, room: Room, socket: SocketRef, user_id: String) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(room).or_insert_with(VecDeque::new);
        session.push_back((socket, user_id));
    }

    pub async fn remove(&self, room: Room) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&room);
    }
}