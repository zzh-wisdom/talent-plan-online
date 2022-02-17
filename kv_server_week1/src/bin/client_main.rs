use kv_server_week1::protos;
use kv_server_week1::Client;

use protos::kvserver::ResponseStatus;
use std::io;

// Key size limit, unit byte
const KEY_LEN: usize = 256;
// Value size limit, unit byte
const VALUE_LEN: usize = 4 * 1024;

/// Client interface
/// Provide a series of operations on the database
/// Provide text-based menu with intuitive tips and result feedback
struct ClientOP {
    c: Client,
}

impl ClientOP {
    /// Constructs a new ClientOP
    ///
    /// Need to pass in the IP address and port number,
    /// and correspond to the server's IP address and port number
    pub fn new(host: String, port: u16) -> Self {
        let client = Client::new(host, port);
        ClientOP { c: client }
    }

    /// Input the key from the standard input
    ///
    /// Read by row, key's size is limited by KEY_LEN
    /// const KEY_LEN: usize = 256;
    pub fn input_key() -> String {
        let mut key = String::new();
        loop {
            key.clear();
            println!("Input Key: ");
            io::stdin()
                .read_line(&mut key)
                .expect("Failed to read line");
            if key.len() > KEY_LEN {
                println!("Error: Key should be no longer than {} B", KEY_LEN);
                continue;
            } else {
                break;
            }
        }
        key
    }

    /// Input the value from the standard input
    ///
    /// Read by row, value's size is limited by VALUE_LEN
    /// const VALUE_LEN: usize = 4 * 1024;
    pub fn input_value() -> String {
        let mut value = String::new();
        loop {
            value.clear();
            println!("Input Value: ");
            io::stdin()
                .read_line(&mut value)
                .expect("Failed to read line");
            if value.len() > VALUE_LEN {
                println!("--Error: Value should be no longer than {} B", VALUE_LEN);
                continue;
            } else {
                break;
            }
        }
        value
    }

    /// Print text menu
    pub fn display_menu(&self) {
        println!("\n\n                       Menu");
        println!("--------------------------------------------------");
        println!("          1) Get              2) Set");
        println!("          3) Insert           4) Update");
        println!("          5) Delete           6) Scan");
        println!("          7) ScanAll          8) Clear");
        println!("          9) Display Menu    10) Exit");
        println!("--------------------------------------------------");
    }

    /// Client get operation
    /// Get the value of a key
    pub fn get(&self) {
        println!("-----------------------GET------------------------");
        let key = ClientOP::input_key();
        let ret = self.c.get(key.clone());
        match ret.status {
            ResponseStatus::kSuccess => println!("Get key({:?}): {:?}", key, ret.value),
            ResponseStatus::kNotFound => println!("Get key({:?}): Not Found!", key),
            _ => println!("System Error!"),
        }
    }

    /// Client set operation
    /// Set a pair of key-value
    /// If the key does not exist, add a pair of key-value pair.
    /// Otherwise, replace the original value.
    pub fn set(&mut self) {
        println!("-----------------------SET------------------------");
        let key = ClientOP::input_key();
        let value = ClientOP::input_value();
        let ret = self.c.set(key.clone(), value.clone());
        match ret.status {
            ResponseStatus::kSuccess => match ret.empty {
                true => println!("Set key({:?}): value({:?}) new added.", key, value),
                false => println!(
                    "Set key({:?}): value({:?}) => value({:?})",
                    key, ret.old_value, value
                ),
            },
            _ => println!("System Error!"),
        }
    }

    /// Client insert operation
    /// Insert a pair of key-value
    /// If the key does not exist, add a pair of key-value pair.
    /// Otherwise, operation failed.
    pub fn insert(&mut self) {
        println!("----------------------INSERT----------------------");
        let key = ClientOP::input_key();
        let value = ClientOP::input_value();
        let ret = self.c.insert(key.clone(), value.clone());
        match ret.status {
            ResponseStatus::kSuccess => println!("Insert({:?}, {:?}): Success!", key, value),
            ResponseStatus::kFailed => println!(
                "Insert({:?}, {:?}): Failed(the key has existed)!",
                key, value
            ),
            _ => println!("Insert: System Error!"),
        }
    }

    /// Client update operation
    /// Update a pair of key-value
    /// If the key exist, replace the original value.
    /// Otherwise, operation failed.
    pub fn update(&mut self) {
        println!("----------------------UPDATE----------------------");
        let key = ClientOP::input_key();
        let value = ClientOP::input_value();
        let ret = self.c.update(key.clone(), value.clone());
        match ret.status {
            ResponseStatus::kSuccess => println!(
                "Update key({:?}): value({:?}) => value({:?})",
                key, ret.old_value, value
            ),
            ResponseStatus::kFailed => {
                println!("Update({:?}, {:?}): Failed(the key not found)!", key, value)
            }
            _ => println!("Update: System Error!"),
        }
    }

    /// Client delete operation
    /// Delete a pair of key-value
    /// If the key exist, operation success.
    /// Otherwise, operation failed.
    pub fn delete(&mut self) {
        println!("----------------------DELETE----------------------");
        let key = ClientOP::input_key();
        let ret = self.c.delete(key.clone());
        match ret.status {
            ResponseStatus::kSuccess => {
                println!("Delete({:?}, {:?}): Success!", key, ret.delete_value)
            }
            ResponseStatus::kNotFound => {
                println!("Delete key({:?}): Failed(the key not found)!", key)
            }
            _ => println!("Delete: System Error!"),
        }
    }

    /// Client scan operation
    /// Range from start_key to end_key, but does not contain end_key.
    pub fn scan(&mut self) {
        println!("-----------------------SCAN-----------------------");
        println!("Start Key");
        let key_start = ClientOP::input_key();
        println!("End Key");
        let key_end = ClientOP::input_key();
        let ret = self.c.scan(key_start.clone(), key_end.clone());
        match ret.status {
            ResponseStatus::kSuccess => {
                println!(
                    "Scan(key_start({:?}), key_end({:?})): Success!",
                    key_start, key_end
                );
                println!("  KEY: VALUE");
                for (key, value) in ret.key_value {
                    println!("{:?}: {:?}", key, value);
                }
            }
            ResponseStatus::kNotFound => println!(
                "Scan(key_start({:?}), key_end({:?})): Empty!",
                key_start, key_end
            ),
            _ => println!("Scan: System Error!"),
        }
    }

    /// Client scan_all operation
    /// View all data
    pub fn scan_all(&mut self) {
        println!("----------------------SCANALL---------------------");
        let ret = self.c.scan_all();
        match ret.status {
            ResponseStatus::kSuccess => {
                println!("  KEY: VALUE");
                for (key, value) in ret.key_value {
                    println!("  {:?}: {:?}", key, value);
                }
            }
            ResponseStatus::kNotFound => println!("  Empty!"),
            _ => println!("ScanAll: System Error!"),
        }
    }

    /// Client clear operation
    /// Clear the database
    pub fn clear(&mut self) {
        println!("----------------------CLEAR-----------------------");
        let ret = self.c.clear();
        match ret.status {
            ResponseStatus::kSuccess => println!("Clear: Success!"),
            _ => println!("Clear: System Error!"),
        }
    }

    /// run the client
    pub fn run(&mut self) {
        self.display_menu();
        loop {
            println!("##Input: ");
            let mut op = String::new();
            io::stdin().read_line(&mut op).expect("Failed to read line");
            let op: u32 = match op.trim().parse() {
                Ok(num) => num,
                Err(_) => continue,
            };
            match op {
                1 => self.get(),
                2 => self.set(),
                3 => self.insert(),
                4 => self.update(),
                5 => self.delete(),
                6 => self.scan(),
                7 => self.scan_all(),
                8 => self.clear(),
                9 => self.display_menu(),
                10 => break,
                _ => {
                    println!("Error Input!");
                    continue;
                }
            }
        }
    }
}

fn main() {
    let test_host = String::from("127.0.0.1");
    let test_port = 10086;
    let mut client = ClientOP::new(test_host.clone(), test_port);
    client.run();
    println!("Client Shutdown.");
}