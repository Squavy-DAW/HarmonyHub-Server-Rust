use std::collections::{HashSet};
use std::sync::RwLock;

#[derive(Default)]
pub struct NamespaceStore(RwLock<HashSet<String>>);

impl NamespaceStore {
    pub async fn insert(&self, ns: String) {
        self.0.write().unwrap().insert(ns);
    }

    pub async fn remove(&self, ns: String) {
        self.0.write().unwrap().remove(&ns);
    }

    pub async fn get_all(&self) -> HashSet<String> {
        self.0.read().unwrap().clone()
    }
}