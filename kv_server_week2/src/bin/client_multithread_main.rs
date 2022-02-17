extern crate chrono;

use kv_server_week2::protos;
use kv_server_week2::Client;

use chrono::prelude::*;
use protos::kvserver::ResponseStatus;
use std::thread;

/// Create a large number of threads,
/// constantly send set operation requests to the server,
/// test the processing power of the server in case of high concurrency
///
/// You need to open the server before you run the client.

// the number of threads
const THREAD_NUM: u32 = 1000;
// the number of set operation per thread
const SET_NUM: u32 = 9;
// every key's size
const KEY_SIZE: usize = 6;
// every value's size
const VALUE_SIZE: usize = 64;
// IP address of the client connection
const TEST_HOST: &'static str = "127.0.0.1";
// Port number of the client connection
const TEST_PORT: u16 = 10086;

// Convert data of type i32 into a string of length KEY_SIZE
// function: generate key
fn get_key(k: i32) -> String {
    format!("{:>0width$}", k, width = KEY_SIZE)
}

// Convert data of type i32 into a string of length VALUE_SIZE
// function: generate value
fn get_value(v: i32) -> String {
    format!("{:>0width$}", v, width = VALUE_SIZE)
}

fn main() {
    let client = Client::new(TEST_HOST.to_string(), TEST_PORT);
    // Clear the data first
    // to prevent the existing data in the database affecting the test.
    client.clear();

    let mut handles = vec![];
    let start_time = Local::now();

    for i in 0..THREAD_NUM {
        let handle = thread::spawn(move || {
            let client = Client::new(TEST_HOST.clone().to_string(), TEST_PORT);
            for j in 0..SET_NUM {
                let ret = client.set(
                    get_key((i * SET_NUM + j + 100000 * (i % 10)) as i32),
                    get_value((i * THREAD_NUM * SET_NUM + j) as i32),
                );
                match ret.status {
                    ResponseStatus::kSuccess => {}
                    _ => println!("{}:{}:set error!", i, j),
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    client.clear();

    // Print the time that all operations spend.
    let end_time = Local::now();
    let use_time = end_time.timestamp_millis() - start_time.timestamp_millis();
    println!("use:{}ms , {:.2}s", use_time, (use_time as f64) / 1000.0);
    println!("Thread num:{} , every thread set:{}", THREAD_NUM, SET_NUM);
    println!(
        "Key size:{} bytes , Value size:{} bytes",
        KEY_SIZE, VALUE_SIZE
    );
}
