use std::fmt;

use super::service;
use futures::Future;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Interval for waiting for log submission, Unit ms
const WAIT_APPLY_INTERVAL: u64 = 100;
// Interval in which the rpc is sent to the new leader, Unit ms
const WAIT_NEW_LEADER: u64 = 50;

#[macro_export]
macro_rules! c_debug {
    ($($arg: tt)*) => {
        //debug!("Debug_Client[{}:{}]: {}", file!(), line!(),format_args!($($arg)*));
    };
}

// Database operation
enum Op {
    Put(String, String),
    Append(String, String),
}

/// Client Structure
pub struct Clerk {
    /// the name of client
    pub name: String,
    servers: Vec<service::KvClient>,
    // You will have to modify this struct.
    // Save the leader's id at the last communication
    leader_id: Arc<Mutex<usize>>,
    // Operation's index, Uniquely distinguish every operation
    index: Arc<Mutex<u64>>,
}

impl fmt::Debug for Clerk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Clerk").field("name", &self.name).finish()
    }
}

impl Clerk {
    /// Create a new `Clerk`.
    pub fn new(name: String, servers: Vec<service::KvClient>) -> Clerk {
        // You'll have to add code here.
        Clerk {
            name,
            servers,
            leader_id: Arc::new(Mutex::new(0)),
            index: Arc::new(Mutex::new(0)),
        }
    }
    /// Get a new operation index
    pub fn get_index(&self) -> u64 {
        let mut index = self.index.lock().unwrap();
        *index += 1;
        *index
    }

    /// fetch the current value for a key.
    /// returns "" if the key does not exist.
    /// keeps trying forever in the face of all other errors.
    //
    // you can send an RPC with code like this:
    // let reply = self.servers[i].get(args).unwrap();
    pub fn get(&self, key: String) -> String {
        // You will have to modify this function.
        let args = service::GetRequest {
            key,
            index: self.get_index(),
            client_name: self.name.clone(),
        };
        // Locked, in fact, the client can not send requests concurrently
        // Prevents a client from sending requests concurrently, causing large index requests to be processed first,
        // which is in violation of order and lead to error
        let mut i = *self.leader_id.lock().unwrap();
        loop {
            c_debug!("Args Get:{:?} leader_id:{}", args, i);
            let reply = self.servers[i].get(&args).wait();
            match reply {
                Ok(rep) => {
                    c_debug!("Reply Get:{:?}", rep);
                    if rep.err == "Ok" {
                        // Successfully submitted
                        c_debug!("Client:{} Get OK", self.name);
                        if !rep.wrong_leader {
                            // Update leader id
                            c_debug!("Client:{} new leader_id:{}", self.name, i);
                            *self.leader_id.lock().unwrap() = i;
                        }
                        return rep.value;
                    } else if rep.err == "Old index" {
                        panic!("rep.err Old index.");
                    }
                    // If the return is not ok, according to the case of wrong_leader, do the corresponding processing
                    if !rep.wrong_leader {
                        // Correct leader
                        *self.leader_id.lock().unwrap() = i;
                        thread::sleep(Duration::from_millis(WAIT_APPLY_INTERVAL));
                    } else {
                        // Wrong leader
                        i = (i + 1) % self.servers.len(); // Try sending to the next server
                        thread::sleep(Duration::from_millis(WAIT_NEW_LEADER));
                    }
                }
                Err(_) => {
                    i = (i + 1) % self.servers.len(); // Try sending to the next server
                    thread::sleep(Duration::from_millis(WAIT_NEW_LEADER));
                }
            }
        }
    }

    /// shared by Put and Append.
    //
    // you can send an RPC with code like this:
    // let reply = self.servers[i].put_append(args).unwrap();
    fn put_append(&self, op: Op) {
        // You will have to modify this function.
        let (key, value, operation) = match op {
            Op::Put(k, v) => (k, v, service::Op::Put),
            Op::Append(k, v) => (k, v, service::Op::Append),
        };
        let args = service::PutAppendRequest {
            key,
            value,
            op: operation as i32,
            index: self.get_index(),
            client_name: self.name.clone(),
        };
        let mut i = *self.leader_id.lock().unwrap();
        loop {
            c_debug!("Args Put_Append:{:?} leader_id:{}", args, i);
            let reply = self.servers[i].put_append(&args).wait();
            match reply {
                Ok(rep) => {
                    c_debug!("Reply Put_Append:{:?} leader:{}", rep, i);
                    if rep.err == "Ok" {
                        // Successfully submitted
                        c_debug!("Client:{} Put_Append OK", self.name);
                        if !rep.wrong_leader {
                            // Update leader id
                            c_debug!("Client:{} new leader_id:{}", self.name, i);
                            *self.leader_id.lock().unwrap() = i;
                        }
                        return;
                    }
                    // If the return is not ok, according to the case of wrong_leader, do the corresponding processing
                    if !rep.wrong_leader {
                        // Correct leader
                        *self.leader_id.lock().unwrap() = i;
                        thread::sleep(Duration::from_millis(WAIT_APPLY_INTERVAL));
                    } else {
                        // Wrong leader
                        i = (i + 1) % self.servers.len(); // Try sending to the next server
                        thread::sleep(Duration::from_millis(WAIT_NEW_LEADER));
                    }
                }
                Err(_) => {
                    i = (i + 1) % self.servers.len(); // Try sending to the next server
                    thread::sleep(Duration::from_millis(WAIT_NEW_LEADER));
                }
            }
        }
    }

    /// Put operation
    pub fn put(&self, key: String, value: String) {
        self.put_append(Op::Put(key, value))
    }
    /// Append operation
    pub fn append(&self, key: String, value: String) {
        self.put_append(Op::Append(key, value))
    }
}
