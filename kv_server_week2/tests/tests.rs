extern crate chrono;

use chrono::prelude::*;
use kv_server_week2::protos::kvserver::ResponseStatus;
use kv_server_week2::Client;
use kv_server_week2::KvServer;
use std::thread;

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
// The maximum time taken to complete a high concurrent test, Unit seconds
const _HIGH_TIME: f64 = 10.0;

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

#[test]
fn test_correctness_persistence_concurrency() {
    let mut kv_server = KvServer::new(TEST_HOST.to_string(), TEST_PORT);
    kv_server.start();
    let client = Client::new(TEST_HOST.to_string(), TEST_PORT);
    client.clear();

    let mut handles = vec![];
    let start_time = Local::now();

    for i in 0..THREAD_NUM {
        let handle = thread::spawn(move || {
            let client = Client::new(TEST_HOST.to_string(), TEST_PORT);
            for j in 0..SET_NUM {
                let ret = client.set(
                    get_key((SET_NUM * i + j) as i32),
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
    kv_server.stop();

    let end_time = Local::now();
    let use_time_ms = end_time.timestamp_millis() - start_time.timestamp_millis();
    let use_time_s = (use_time_ms as f64) / 1000.0;
    assert!(use_time_s <= _HIGH_TIME);
    println!("use:{}ms , {:.2}s", use_time_ms, use_time_s);
    println!("Thread num:{} , every thread put:{}", THREAD_NUM, SET_NUM);
    println!(
        "Key size:{} bytes , Value size:{} bytes",
        KEY_SIZE, VALUE_SIZE
    );

    let mut kv_server = KvServer::new(TEST_HOST.to_string(), TEST_PORT);
    kv_server.start();
    let client = Client::new(TEST_HOST.to_string(), TEST_PORT);
    client.clear();

    let ret = client.insert("1".to_string(), "aaaa".to_string());
    assert_eq!(ret.status, ResponseStatus::kSuccess);

    let ret = client.get("1".to_string());
    assert_eq!(ret.status, ResponseStatus::kSuccess);

    client.set("1".to_string(), "aaaaa".to_string());
    client.set("2".to_string(), "bbbbb".to_string());
    client.set("3".to_string(), "cc".to_string());
    client.set("4".to_string(), "ee".to_string());

    let ret = client.insert("1".to_string(), "aaaa".to_string());
    assert_eq!(ret.status, ResponseStatus::kFailed);

    let ret = client.update("5".to_string(), "ccccc".to_string());
    assert_eq!(ret.status, ResponseStatus::kFailed);

    let ret = client.update("3".to_string(), "ccccc".to_string());
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.old_value, "cc");

    let ret = client.scan_all();
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.key_value.len(), 4);

    let ret = client.delete("5".to_string());
    assert_eq!(ret.status, ResponseStatus::kNotFound);

    let ret = client.delete("4".to_string());
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.delete_value, "ee");

    let ret = client.scan_all();
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.key_value.len(), 3);

    let ret = client.scan("1".to_string(), "3".to_string());
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.key_value.len(), 2);

    let ret = client.scan("1".to_string(), "5".to_string());
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.key_value.len(), 3);

    client.clear();

    let ret = client.scan_all();
    assert_eq!(ret.status, ResponseStatus::kNotFound);

    // Insert some special characters to verify correctness
    client.set("".to_string(), "".to_string());
    client.set(" ".to_string(), " ".to_string());
    client.set("\t1\n".to_string(), "aa a\naa".to_string());
    client.set(" 2\n".to_string(), "bb\tb\nbb".to_string());
    client.set("3\03".to_string(), "c\0ccc\nc".to_string());
    client.set("4".to_string(), "d\tddd\nd".to_string());
    client.set("5".to_string(), "e eee\ne".to_string());
    client.set("6".to_string(), "f\0fff\nf".to_string());

    kv_server.stop();

    // Test persistence
    let mut kv_server = KvServer::new("127.0.0.1".to_string(), 10086);
    kv_server.start();

    let ret = client.scan_all();
    assert_eq!(ret.status, ResponseStatus::kSuccess);
    assert_eq!(ret.key_value.len(), 8);

    let ret = client.get("".to_string());
    assert_eq!(ret.value, "");
    let ret = client.get(" ".to_string());
    assert_eq!(ret.value, " ");
    let ret = client.get("\t1\n".to_string());
    assert_eq!(ret.value, "aa a\naa");
    let ret = client.get(" 2\n".to_string());
    assert_eq!(ret.value, "bb\tb\nbb");
    let ret = client.get("3\03".to_string());
    assert_eq!(ret.value, "c\0ccc\nc");

    kv_server.stop();
}
