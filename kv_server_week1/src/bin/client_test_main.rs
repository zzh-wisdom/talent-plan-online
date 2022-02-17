use kv_server_week1::protos;
use kv_server_week1::Client;

use protos::kvserver::ResponseStatus;

/// The client to perform a series of operations.
/// Print the information returned by each operation,
/// observe the running of the server,
/// and help determine the correctness of the program.
///
/// You need to open the server before you run the client.
fn main() {
    let test_host = String::from("127.0.0.1");
    let test_port = 10086;

    let client = Client::new(test_host.clone(), test_port);

    client.insert("1".to_string(), "aaaa".to_string());
    println!("Insert(1, aaaa)");
    let ret = client.get("1".to_string());
    match ret.status {
        ResponseStatus::kSuccess => println!("Get key(1): {}", ret.value),
        ResponseStatus::kNotFound => println!("Get key(1): Not Found!"),
        _ => println!("System Error!"),
    }

    client.set("1".to_string(), "aaaaa".to_string());
    client.set("2".to_string(), "bbbbb".to_string());
    client.set("3".to_string(), "cc".to_string());
    client.set("4".to_string(), "ee".to_string());
    println!("Set(1, aaaaa)");
    println!("Set(2, bbbbb)");
    println!("Set(3, cc)");
    println!("Set(4, ee)");

    // Insert a key-value pair whose key already exists.
    let ret = client.insert("1".to_string(), "aaaa".to_string());
    match ret.status {
        ResponseStatus::kSuccess => println!("Insert(1, aaaa): Success!"),
        ResponseStatus::kFailed => println!("Insert(1, aaaa): Failed(the key has existed)!"),
        _ => println!("Insert: System Error!"),
    }

    // Update a key-value pair whose key does not exists.
    let ret = client.update("5".to_string(), "ccccc".to_string());
    match ret.status {
        ResponseStatus::kSuccess => {
            println!("Update key(5): value({}) => value(ccccc)", ret.old_value)
        }
        ResponseStatus::kFailed => println!("Update(5, ccccc): Failed(the key not found)!"),
        _ => println!("Update: System Error!"),
    }

    // Update a key-value pair whose key already exists.
    let ret = client.update("3".to_string(), "ccccc".to_string());
    match ret.status {
        ResponseStatus::kSuccess => {
            println!("Update key(3): value({}) => value(ccccc)", ret.old_value)
        }
        ResponseStatus::kFailed => println!("Update(3, ccccc): Failed(the key not found)!"),
        _ => println!("Update: System Error!"),
    }

    let ret = client.scan_all();
    match ret.status {
        ResponseStatus::kSuccess => println!("ScanAll{{ {:?} }}", ret.key_value),
        ResponseStatus::kNotFound => println!("ScanAll None"),
        _ => println!("ScanAll: System Error!"),
    }

    // Delete a key-value pair whose key does not exists.
    let ret = client.delete("5".to_string());
    match ret.status {
        ResponseStatus::kSuccess => println!("Delete(5, {}): Success!", ret.delete_value),
        ResponseStatus::kNotFound => println!("Delete key(5): Failed(the key not found)!"),
        _ => println!("Delete: System Error!"),
    }
    // Delete a key-value pair whose key already exists.
    let ret = client.delete("4".to_string());
    match ret.status {
        ResponseStatus::kSuccess => println!("Delete(4, {}): Success!", ret.delete_value),
        ResponseStatus::kNotFound => println!("Delete key(4): Failed(the key not found)!"),
        _ => println!("Delete: System Error!"),
    }

    let ret = client.scan_all();
    match ret.status {
        ResponseStatus::kSuccess => println!("ScanAll{{ {:?} }}", ret.key_value),
        ResponseStatus::kNotFound => println!("ScanAll None!"),
        _ => println!("ScanAll: System Error!"),
    }

    let ret = client.scan("1".to_string(), "3".to_string());
    match ret.status {
        ResponseStatus::kSuccess => {
            println!("Scan[1, 3)");
            println!("Scan{{ {:?} }}", ret.key_value)
        }
        ResponseStatus::kNotFound => println!("Scan[1, 3): None!"),
        _ => println!("Scan: System Error!"),
    }

    let ret = client.scan("1".to_string(), "5".to_string());
    match ret.status {
        ResponseStatus::kSuccess => {
            println!("Scan[1, 5)");
            println!("Scan{{ {:?} }}", ret.key_value)
        }
        ResponseStatus::kNotFound => println!("Scan[1, 5): None!"),
        _ => println!("Scan: System Error!"),
    }

    client.clear();
    println!("Clear");

    let ret = client.scan_all();
    match ret.status {
        ResponseStatus::kSuccess => println!("ScanAll{{ {:?} }}", ret.key_value),
        ResponseStatus::kNotFound => println!("ScanAll None!"),
        _ => println!("ScanAll: System Error!"),
    }

    // Insert some special characters to verify correctness
    client.set("".to_string(), "".to_string());
    client.set(" ".to_string(), " ".to_string());
    client.set("\t1\n".to_string(), "aa a\naa".to_string());
    client.set(" 2\n".to_string(), "bb\tb\nbb".to_string());
    client.set("3\03".to_string(), "c\0ccc\nc".to_string());
    client.set("4".to_string(), "d\tddd\nd".to_string());
    client.set("5".to_string(), "e eee\ne".to_string());
    client.set("6".to_string(), "f\0fff\nf".to_string());

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
}
