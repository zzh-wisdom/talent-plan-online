extern crate futures;

use futures::sync::oneshot;
use futures::Future;
use kv_server_week2::KvServer;
use std::{io, thread};

/// Run a kv server.
/// the listening IP address is 127.0.0.1.
/// The port number is 10086.
///
/// Input "exit" can close the server normally.

fn main() {
    let mut kv_server = KvServer::new("127.0.0.1".to_string(), 10086);
    kv_server.start();

    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Input \"exit\" to shutdown server...");
        loop {
            let mut input = String::new();
            let ret = io::stdin().read_line(&mut input);
            if let Err(e) = ret {
                println!("Error: {}", e);
                continue;
            }
            if input.trim().eq("exit") {
                tx.send(()).unwrap();
                break;
            }
        }
    });
    let _ = rx.wait();
}
