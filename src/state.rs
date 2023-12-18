// use std::collections::{HashMap, VecDeque};
// use socketioxide::adapter::Room;
// use socketioxide::extract::SocketRef;
// use tokio::sync::RwLock;
//
// pub type SessionStore = HashMap<Room, VecDeque<(SocketRef, String)>>; // session -> [socket, user_id]
//
// #[derive(Default)]
// pub struct State {
//     pub sessions: RwLock<SessionStore>
// }
//
// impl State {
//     pub async fn insert(&self, room: Room, socket: SocketRef, user_id: String) {
//         let mut sessions = self.sessions.write().await;
//         let session = sessions.entry(room).or_insert_with(VecDeque::new);
//         session.push_back((socket, user_id));
//     }
//
//     pub async fn remove(&self, room: Room) {
//         let mut sessions = self.sessions.write().await;
//         sessions.remove(&room);
//     }
// }

use std::collections::{HashMap, VecDeque};
use tokio::sync::RwLock;

#[derive(serde::Serialize, Clone, Debug)]
pub struct Message {
    pub text: String,
    pub user: String,
}

pub type RoomStore = HashMap<String, VecDeque<Message>>;

#[derive(Default)]
pub struct MessageStore {
    pub messages: RwLock<RoomStore>,
}

impl MessageStore {
    pub async fn insert(&self, room: &str, message: Message) {
        let mut binding = self.messages.write().await;
        let messages = binding.entry(room.to_owned()).or_default();
        messages.push_front(message);
        messages.truncate(20);
    }

    pub async fn get(&self, room: &str) -> Vec<Message> {
        let messages = self.messages.read().await.get(room).cloned();
        messages.unwrap_or_default().into_iter().rev().collect()
    }
}