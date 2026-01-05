//! MyLib v2.0.0 - A sample library for demonstrating API upgrades.
//!
//! Breaking changes from v1.0.0:
//! - `User` renamed to `UserAccount` with new `email` field
//! - `Status` renamed to `ConnectionStatus`
//! - `get_user` renamed to `fetch_user`
//! - `process_data` renamed to `transform_data`
//! - `connect` now requires `port` and `timeout` parameters
//! - `save` no longer takes `sync` parameter
//! - `query` parameters reordered from (a, b, c) to (c, a, b)
//! - `find` now returns `Option<Item>` instead of `Item`
//! - `parse` now returns `ParseResult` instead of `Result<String, String>`
//! - `deprecated_fn` has been removed
//! - `utils` module renamed to `helpers`

use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

/// A user account in the system (renamed from User).
#[derive(Debug, Clone)]
pub struct UserAccount {
    pub id: u64,
    pub name: String,
    pub email: String, // New field in v2
}

impl UserAccount {
    pub fn new(id: u64, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }
}

/// Connection status (renamed from Status).
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

/// Configuration for the library (with new port field).
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16, // New field in v2
}

impl Config {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }
}

/// Database query result item.
#[derive(Debug, Clone)]
pub struct Item {
    pub key: String,
    pub value: String,
}

/// Result of parsing operations (new in v2).
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub output: String,
    pub warnings: Vec<String>,
}

/// Fetch a user by their ID (renamed from get_user).
pub fn fetch_user(id: u64) -> Option<UserAccount> {
    if id > 0 {
        Some(UserAccount::new(
            id,
            &format!("User{}", id),
            &format!("user{}@example.com", id),
        ))
    } else {
        None
    }
}

/// Transform raw data bytes (renamed from process_data).
pub fn transform_data(data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b.wrapping_add(1)).collect()
}

/// Connect to a host with port and timeout (signature changed).
pub fn connect(host: &str, port: u16, timeout: Duration) -> Result<ConnectionStatus, io::Error> {
    if host.is_empty() {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "empty host"))
    } else if port == 0 {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid port"))
    } else {
        let _ = timeout;
        Ok(ConnectionStatus::Connected)
    }
}

/// Save data (sync parameter removed).
pub fn save(data: &str) -> Result<(), io::Error> {
    io::stdout().flush()?;
    let _ = data;
    Ok(())
}

/// Query with reordered parameters (c, a, b instead of a, b, c).
pub fn query(c: bool, a: &str, b: i32) -> Vec<Item> {
    let mut results = Vec::new();
    if c {
        results.push(Item {
            key: a.to_string(),
            value: b.to_string(),
        });
    }
    results
}

/// Find an item by key (now returns Option<Item>).
pub fn find(key: &str) -> Option<Item> {
    if key.is_empty() {
        None
    } else {
        Some(Item {
            key: key.to_string(),
            value: "found".to_string(),
        })
    }
}

/// Parse a string into a ParseResult (return type changed).
pub fn parse(input: &str) -> ParseResult {
    if input.is_empty() {
        ParseResult {
            output: String::new(),
            warnings: vec!["empty input".to_string()],
        }
    } else {
        ParseResult {
            output: input.to_uppercase(),
            warnings: Vec::new(),
        }
    }
}

// Note: deprecated_fn has been removed in v2

/// Helper functions module (renamed from utils).
pub mod helpers {
    use super::*;

    /// A helper function that returns a greeting.
    pub fn helper() -> String {
        "Hello from helpers".to_string()
    }

    /// Format a user account for display.
    pub fn format_user(user: &UserAccount) -> String {
        format!(
            "UserAccount(id={}, name={}, email={})",
            user.id, user.name, user.email
        )
    }

    /// Create a config map.
    pub fn create_config_map() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("version".to_string(), "2.0.0".to_string());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_user() {
        let user = fetch_user(1).unwrap();
        assert_eq!(user.id, 1);
        assert!(!user.email.is_empty());
    }

    #[test]
    fn test_transform_data() {
        let result = transform_data(&[1, 2, 3]);
        assert_eq!(result, vec![2, 3, 4]);
    }

    #[test]
    fn test_connect() {
        assert!(connect("localhost", 8080, Duration::from_secs(30)).is_ok());
        assert!(connect("", 8080, Duration::from_secs(30)).is_err());
        assert!(connect("localhost", 0, Duration::from_secs(30)).is_err());
    }

    #[test]
    fn test_find() {
        assert!(find("key").is_some());
        assert!(find("").is_none());
    }
}
