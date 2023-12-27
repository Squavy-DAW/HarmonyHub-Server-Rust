use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClientId(pub String);

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