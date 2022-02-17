use super::service::*;
use crate::raft;

use futures::stream::Stream;
use futures::sync::mpsc::{unbounded, UnboundedReceiver};
use futures::Async;
use labrpc::RpcFuture;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;

#[macro_export]
macro_rules! s_debug {
    ($($arg: tt)*) => {
        //debug!("Debug_Server[{}:{}]: {}", file!(), line!(),format_args!($($arg)*));
    };
}

/// Save the last reply
#[derive(Clone, PartialEq, Message)]
pub struct LatestReply {
    /// The index of the request that has been replied last time
    #[prost(uint64, tag = "1")]
    pub index: u64,

    /// Save the result of the `get` operation
    #[prost(string, tag = "2")]
    pub value: String,
}

/// Log entry
#[derive(Clone, PartialEq, Message)]
pub struct Entry {
    // operation index
    #[prost(uint64, tag = "1")]
    index: u64,
    #[prost(string, tag = "2")]
    /// the name of client
    pub client_name: String,
    #[prost(uint64, tag = "3")]
    /// 1 mean get, 2 mean put, 3 mean append
    pub op: u64,
    #[prost(string, tag = "4")]
    pub key: String,
    #[prost(string, tag = "5")]
    pub value: String,
}

pub struct KvServer {
    pub rf: raft::Node,
    me: usize,
    // snapshot if log grows this big
    maxraftstate: Option<usize>,
    // Your definitions here.
    pub apply_ch: UnboundedReceiver<raft::ApplyMsg>,

    /// Storage database
    pub db: BTreeMap<String, String>,
    /// Save the last reply of each client
    pub lastest_reply: HashMap<String, LatestReply>,
}

impl KvServer {
    /// Create a new `KvServer`.
    pub fn new(
        servers: Vec<raft::service::RaftClient>,
        me: usize,
        persister: Box<dyn raft::persister::Persister>,
        maxraftstate: Option<usize>,
    ) -> KvServer {
        // You may need initialization code here.
        let (tx, apply_ch) = unbounded();
        let rf = raft::Raft::new(servers, me, persister, tx);

        KvServer {
            me,
            maxraftstate,
            rf: raft::Node::new(rf),
            apply_ch,
            db: BTreeMap::new(),
            lastest_reply: HashMap::new(),
        }
    }
    /// Get raft state
    pub fn get_state(&self) -> raft::State {
        self.rf.get_state()
    }
}

// Choose concurrency paradigm.
//
// You can either drive the kv server by the rpc framework,
//
// ```rust
// struct Node { server: Arc<Mutex<KvServer>> }
// ```
//
// or spawn a new thread runs the kv server and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    // Your definitions here.
    /// peer id, for debugging
    pub me: usize,
    server: Arc<Mutex<KvServer>>,
    // The thread that will apply the log
    apply_thread_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    // On-off status
    close_status: Arc<Mutex<bool>>,
}

impl Node {
    /// Create a new `Node`.
    pub fn new(kv: KvServer) -> Node {
        // Your code here.
        let node = Node {
            me: kv.me,
            server: Arc::new(Mutex::new(kv)),
            apply_thread_handle: Arc::new(Mutex::new(None)),
            close_status: Arc::new(Mutex::new(false)),
        };
        node.create_apply_thread();
        node
    }
    /// Initially, create logs apply thread
    pub fn create_apply_thread(&self) {
        let node = self.clone();
        let handle = thread::spawn(move || {
            loop {
                if *node.close_status.lock().unwrap() == true {
                    s_debug!("server:{} apply_thread close.", node.me);
                    break;
                }
                if let Ok(Async::Ready(Some(apply_msg))) =
                    futures::executor::spawn(futures::lazy(|| {
                        node.server.lock().unwrap().apply_ch.poll()
                    }))
                    .wait_future()
                {
                    if !apply_msg.command_valid || apply_msg.command.is_empty() {
                        // Invalid, or the log is empty
                        continue;
                    }
                    let mut server = node.server.lock().unwrap();
                    let command_index = apply_msg.command_index;
                    let entry: Entry = match labcodec::decode(&apply_msg.command) {
                        Ok(en) => en,
                        Err(e) => {
                            s_debug!("Decode Error: {:?}", e);
                            continue;
                        }
                    };
                    s_debug!("server:{} Entry:{:?}", node.me, entry);
                    if server.lastest_reply.get(&entry.client_name).is_none()
                        || server.lastest_reply.get(&entry.client_name).unwrap().index < entry.index
                    {
                        // Update data
                        let mut lastest_reply = LatestReply {
                            index: entry.index,
                            value: String::new(),
                        };
                        match entry.op {
                            1 => {
                                // get
                                if server.db.get(&entry.key).is_some() {
                                    lastest_reply.value =
                                        server.db.get(&entry.key).unwrap().clone();
                                }
                                server
                                    .lastest_reply
                                    .insert(entry.client_name.clone(), lastest_reply.clone());
                                s_debug!(
                                    "server:{} client:{} apply:{:?}",
                                    server.me,
                                    entry.client_name,
                                    lastest_reply
                                );
                            }
                            2 => {
                                // put
                                server.db.insert(entry.key.clone(), entry.value.clone());
                                server
                                    .lastest_reply
                                    .insert(entry.client_name.clone(), lastest_reply.clone());
                                s_debug!(
                                    "server:{} client:{} apply:{:?}",
                                    server.me,
                                    entry.client_name,
                                    lastest_reply
                                );
                            }
                            3 => {
                                // append
                                if let Some(v) = server.db.get_mut(&entry.key) {
                                    s_debug!("append Before key:{} value:{}", entry.key, *v);
                                    v.push_str(&entry.value.clone());
                                    s_debug!("append after key:{} value:{}", entry.key, *v);
                                } else {
                                    server.db.insert(entry.key.clone(), entry.value.clone());
                                }
                                server
                                    .lastest_reply
                                    .insert(entry.client_name.clone(), lastest_reply.clone());
                                s_debug!(
                                    "server:{} client:{} apply:{:?}",
                                    server.me,
                                    entry.client_name,
                                    lastest_reply
                                );
                            }
                            _ => {
                                // error
                                s_debug!("Error:apply op:{:?} error", entry);
                                continue;
                            }
                        }
                    }
                }
            }
        });
        *self.apply_thread_handle.lock().unwrap() = Some(handle);
    }

    /// the tester calls Kill() when a KVServer instance won't
    /// be needed again. you are not required to do anything
    /// in Kill(), but it might be convenient to (for example)
    /// turn off debug output from this instance.
    pub fn kill(&self) {
        // Your code here, if desired.
        self.server.lock().unwrap().rf.kill();
        *self.close_status.lock().unwrap() = true;
        let apply_thread_handle = self.apply_thread_handle.lock().unwrap().take();
        if apply_thread_handle.is_some() {
            let _ = apply_thread_handle.unwrap().join();
        }
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.get_state().term()
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.get_state().is_leader()
    }
    /// Get raft state
    pub fn get_state(&self) -> raft::State {
        // Your code here.
        self.server.lock().unwrap().get_state()
    }
}

impl KvService for Node {
    fn get(&self, arg: GetRequest) -> RpcFuture<GetReply> {
        // Your code here.
        s_debug!("Server:{} get:{:?}", self.me, arg);
        let mut reply = GetReply {
            wrong_leader: true,
            err: String::new(),
            value: String::new(),
        };
        if !self.is_leader() {
            return Box::new(futures::future::result(Ok(reply)));
        }
        s_debug!("Info: server:{} get for lock...", self.me);
        let server = self.server.lock().unwrap();
        s_debug!("Info: server:{} have locked.", self.me);
        if let Some(re) = server.lastest_reply.get(&arg.client_name) {
            if arg.index < re.index {
                reply.err = String::from("Old index");
                s_debug!(
                    "Warning:server[{}:{}] client[{}:{}] get Old index.",
                    self.me,
                    re.index,
                    arg.client_name,
                    arg.index
                );
                return Box::new(futures::future::result(Ok(reply)));
            } else if arg.index == re.index {
                // return the result
                reply.wrong_leader = false;
                reply.err = String::from("Ok");
                reply.value = re.value.clone();
                s_debug!(
                    "Info:server[{}:{}] client[{}:{}] get Equal index.",
                    self.me,
                    re.index,
                    arg.client_name,
                    arg.index,
                );
                return Box::new(futures::future::result(Ok(reply)));
            }
        }
        let command = Entry {
            index: arg.index,
            client_name: arg.client_name.clone(),
            op: 1,
            key: arg.key.clone(),
            value: String::new(),
        };
        match server.rf.start(&command) {
            // send log to raft
            Ok((index, term)) => {
                reply.wrong_leader = false;
                s_debug!(
                    "Info: server:{} client:{} start:{:?}",
                    self.me,
                    arg.client_name,
                    command
                );
                Box::new(futures::future::result(Ok(reply)))
            }
            Err(_) => {
                reply.wrong_leader = true;
                s_debug!(
                    "Error: server:{} client:{} start:{:?} error.",
                    self.me,
                    arg.client_name,
                    command
                );
                Box::new(futures::future::result(Ok(reply)))
            }
        }
    }

    fn put_append(&self, arg: PutAppendRequest) -> RpcFuture<PutAppendReply> {
        // Your code here.
        s_debug!("Info: server:{} put_append:{:?}", self.me, arg);
        let mut reply = PutAppendReply {
            wrong_leader: true,
            err: String::new(),
        };
        if !self.is_leader() {
            return Box::new(futures::future::result(Ok(reply)));
        }
        s_debug!("Info: server:{} put_append for lock...", self.me);
        let server = self.server.lock().unwrap();
        s_debug!("Info: server:{} put_append have locked.", self.me);
        if let Some(re) = server.lastest_reply.get(&arg.client_name) {
            if arg.index < re.index {
                reply.err = String::from("Ok");
                s_debug!(
                    "Warning:server[{}:{}] client[{}:{}] put_append Old index",
                    self.me,
                    re.index,
                    arg.client_name,
                    arg.index
                );
                return Box::new(futures::future::result(Ok(reply)));
            } else if arg.index == re.index {
                reply.wrong_leader = false;
                reply.err = String::from("Ok");
                s_debug!(
                    "Info: server:{} client:{} put_append Equal index",
                    self.me,
                    arg.client_name,
                );
                return Box::new(futures::future::result(Ok(reply)));
            }
        }
        let command = Entry {
            index: arg.index,
            client_name: arg.client_name.clone(),
            op: (arg.op + 1) as u64,
            key: arg.key.clone(),
            value: arg.value.clone(),
        };
        match server.rf.start(&command) {
            // send log to raft
            Ok((index, term)) => {
                reply.wrong_leader = false;
                s_debug!(
                    "Info: server:{} client:{} start:{:?}",
                    self.me,
                    arg.client_name,
                    command
                );
                Box::new(futures::future::result(Ok(reply)))
            }
            Err(_) => {
                reply.wrong_leader = true;
                s_debug!(
                    "Error: server:{} client:{} start:{:?} error.",
                    self.me,
                    arg.client_name,
                    command
                );
                Box::new(futures::future::result(Ok(reply)))
            }
        }
    }
}
