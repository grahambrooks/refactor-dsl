//! MyLib v1.0.0 - A sample library for demonstrating API upgrades.

use std::collections::HashMap;
use std::io::{self, Write};

/// A user in the system.
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
}

impl User {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

/// Connection status.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Connected,
    Disconnected,
    Error(String),
}

/// Configuration for the library.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
}

impl Config {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
        }
    }
}

/// Database query result item.
#[derive(Debug, Clone)]
pub struct Item {
    pub key: String,
    pub value: String,
}

/// Get a user by their ID.
pub fn get_user(id: u64) -> Option<User> {
    // Simulated user lookup
    if id > 0 {
        Some(User::new(id, &format!("User{}", id)))
    } else {
        None
    }
}

/// Process raw data bytes.
pub fn process_data(data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b.wrapping_add(1)).collect()
}

/// Connect to a host.
pub fn connect(host: &str) -> Result<Status, io::Error> {
    if host.is_empty() {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "empty host"))
    } else {
        Ok(Status::Connected)
    }
}

/// Save data with optional sync.
pub fn save(data: &str, sync: bool) -> Result<(), io::Error> {
    if sync {
        io::stdout().flush()?;
    }
    let _ = data;
    Ok(())
}

/// Query with multiple parameters.
pub fn query(a: &str, b: i32, c: bool) -> Vec<Item> {
    let mut results = Vec::new();
    if c {
        results.push(Item {
            key: a.to_string(),
            value: b.to_string(),
        });
    }
    results
}

/// Find an item by key.
pub fn find(key: &str) -> Item {
    Item {
        key: key.to_string(),
        value: "found".to_string(),
    }
}

/// Parse a string into a result.
pub fn parse(input: &str) -> Result<String, String> {
    if input.is_empty() {
        Err("empty input".to_string())
    } else {
        Ok(input.to_uppercase())
    }
}

/// This function is deprecated and will be removed.
#[deprecated(note = "Use fetch_user instead")]
pub fn deprecated_fn() {
    println!("This function is deprecated");
}

/// Utility functions module.
pub mod utils {
    use super::*;

    /// A helper function that returns a greeting.
    pub fn helper() -> String {
        "Hello from utils".to_string()
    }

    /// Format a user for display.
    pub fn format_user(user: &User) -> String {
        format!("User(id={}, name={})", user.id, user.name)
    }

    /// Create a config map.
    pub fn create_config_map() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("version".to_string(), "1.0.0".to_string());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user() {
        let user = get_user(1).unwrap();
        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_process_data() {
        let result = process_data(&[1, 2, 3]);
        assert_eq!(result, vec![2, 3, 4]);
    }

    #[test]
    fn test_connect() {
        assert!(connect("localhost").is_ok());
        assert!(connect("").is_err());
    }
}
