use std::collections::HashSet;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use socketioxide::ack::AckResponse;

pub type ClientId = String;

pub fn create_random_namespace(namespaces: &HashSet<String>) -> String {
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

pub trait AckResponseExt {
    fn transform_response(&self) -> (Value, Vec<Vec<u8>>);
}

impl AckResponseExt for AckResponse<Value> {
    fn transform_response(&self) -> (Value, Vec<Vec<u8>>) {
        let target_id = self.socket.extensions.get::<ClientId>().unwrap().clone();
        let mut data = Map::new();
        data.insert("id".to_string(), Value::String(target_id));
        data.insert("data".to_string(), self.data.clone());
        (data.into(), self.binary.clone())
    }
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