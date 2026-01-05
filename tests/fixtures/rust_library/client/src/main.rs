//! Client application using mylib v1.0.0
//!
//! This client needs to be upgraded to work with mylib v2.0.0.
//! The upgrade analyzer should detect all the necessary changes.

// Simulating imports from mylib v1
// use mylib::{User, Status, Config, Item};
// use mylib::{get_user, process_data, connect, save, query, find, parse, deprecated_fn};
// use mylib::utils;

/// Simulated User struct (from v1)
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
}

/// Simulated Status enum (from v1)
#[derive(Debug)]
pub enum Status {
    Connected,
    Disconnected,
}

/// Simulated Config struct (from v1)
pub struct Config {
    pub host: String,
}

/// Simulated Item struct
pub struct Item {
    pub key: String,
    pub value: String,
}

// Simulated library functions (these represent calls to mylib v1)
fn get_user(id: u64) -> Option<User> {
    Some(User {
        id,
        name: format!("User{}", id),
    })
}

fn process_data(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

fn connect(host: &str) -> Result<Status, std::io::Error> {
    let _ = host;
    Ok(Status::Connected)
}

fn save(data: &str, sync: bool) -> Result<(), std::io::Error> {
    let _ = (data, sync);
    Ok(())
}

fn query(a: &str, b: i32, c: bool) -> Vec<Item> {
    let _ = (a, b, c);
    Vec::new()
}

fn find(key: &str) -> Item {
    Item {
        key: key.to_string(),
        value: "found".to_string(),
    }
}

fn parse(input: &str) -> Result<String, String> {
    Ok(input.to_uppercase())
}

#[allow(deprecated)]
fn deprecated_fn() {
    println!("deprecated");
}

mod utils {
    pub fn helper() -> String {
        "helper".to_string()
    }
}

fn main() {
    println!("=== Client Application ===\n");

    // Using get_user (should become fetch_user)
    let user: User = get_user(42).expect("User not found");
    println!("Found user: {:?}", user);

    // Using another get_user call
    if let Some(admin) = get_user(1) {
        println!("Admin: {:?}", admin);
    }

    // Using process_data (should become transform_data)
    let data = vec![1u8, 2, 3, 4, 5];
    let processed = process_data(&data);
    println!("Processed data: {:?}", processed);

    // Using connect (needs port and timeout parameters)
    let config = Config {
        host: "localhost".to_string(),
    };
    match connect(&config.host) {
        Ok(Status::Connected) => println!("Connected to {}", config.host),
        Ok(Status::Disconnected) => println!("Disconnected"),
        Err(e) => println!("Error: {}", e),
    }

    // Using save (sync parameter should be removed)
    save("important data", true).expect("Failed to save");
    save("more data", false).expect("Failed to save");

    // Using query (parameters should be reordered)
    let results = query("search", 10, true);
    println!("Query results: {} items", results.len());

    // Another query call
    let more_results = query("filter", 5, false);
    println!("More results: {} items", more_results.len());

    // Using find (should return Option<Item> now)
    let item: Item = find("my_key");
    println!("Found item: {}", item.key);

    // Using parse (return type changed)
    match parse("hello world") {
        Ok(result) => println!("Parsed: {}", result),
        Err(e) => println!("Parse error: {}", e),
    }

    // Using deprecated_fn (should be removed)
    deprecated_fn();

    // Using utils module (should become helpers)
    let greeting = utils::helper();
    println!("Greeting: {}", greeting);

    println!("\n=== Done ===");
}

/// A service that uses the library
pub struct UserService {
    config: Config,
}

impl UserService {
    pub fn new(host: &str) -> Self {
        Self {
            config: Config {
                host: host.to_string(),
            },
        }
    }

    pub fn get_user_by_id(&self, id: u64) -> Option<User> {
        // Uses get_user internally
        get_user(id)
    }

    pub fn process_user_data(&self, data: &[u8]) -> Vec<u8> {
        // Uses process_data internally
        process_data(data)
    }

    pub fn connect_to_server(&self) -> Result<Status, std::io::Error> {
        // Uses connect internally
        connect(&self.config.host)
    }

    pub fn save_user(&self, user: &User) -> Result<(), std::io::Error> {
        // Uses save internally
        let data = format!("{}:{}", user.id, user.name);
        save(&data, true)
    }

    pub fn search_users(&self, query_str: &str, limit: i32) -> Vec<Item> {
        // Uses query internally
        query(query_str, limit, true)
    }

    pub fn find_user_item(&self, key: &str) -> Item {
        // Uses find internally
        find(key)
    }

    pub fn get_helper_message(&self) -> String {
        // Uses utils::helper internally
        utils::helper()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_service() {
        let service = UserService::new("localhost");
        assert!(service.get_user_by_id(1).is_some());
    }

    #[test]
    fn test_get_user() {
        let user = get_user(42).unwrap();
        assert_eq!(user.id, 42);
    }

    #[test]
    fn test_process_data() {
        let result = process_data(&[1, 2, 3]);
        assert_eq!(result.len(), 3);
    }
}
