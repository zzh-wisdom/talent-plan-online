use futures::sync::mpsc::UnboundedSender;
use futures::Future;
use labcodec;
use labrpc::RpcFuture;

use rand::Rng;
use std::cmp;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;
pub mod service;
#[cfg(test)]
mod tests;

use self::errors::*;
use self::persister::*;
use self::service::*;

#[macro_export]
macro_rules! my_debug {
    ($($arg: tt)*) => {
        //debug!("Debug[{}:{}]: {}", file!(), line!(),format_args!($($arg)*));
    };
}

/// the lower limit of timeout thread's timeout period, unit ms
const TIMEOUT_LOW_MIC: u64 = 250;
/// the upper limit of timeout thread's timeout period, unit ms
const TIMEOUT_HIGH_MIC: u64 = 350;
/// The interval at which the leader sends the log/heartbeat, unit ms
const HEARTBEAT_INTERVAL: u64 = 35;
/// The maximum number of logs sent each time
const SEND_ENTRIES_MAX: u64 = 128;

/// The information structure of the raft submission log to the upper layer
pub struct ApplyMsg {
    pub command_valid: bool,
    pub command: Vec<u8>,
    pub command_index: u64,
}

/// Log format during testing, used for debugging
#[derive(Clone, PartialEq, Message)]
pub struct Entry {
    #[prost(uint64, tag = "100")]
    pub x: u64,
}

/// State of a raft peer.
#[derive(Default, Clone, Debug)]
pub struct State {
    /// Term, monotonically increasing
    pub term: u64,
    /// Leader mark
    pub is_leader: bool,
    /// Follower mark
    pub is_follower: bool,
}

impl State {
    /// Constructs a new `State`.
    /// Initially, it is follower. Term is zero
    pub fn new() -> State {
        State {
            term: 0,
            is_leader: false,
            is_follower: true,
        }
    }
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.term
    }
    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
    /// Whether this peer believes it is the follower.
    pub fn is_follower(&self) -> bool {
        self.is_follower
    }
}

/// The structure of the log stored in the Raft
#[derive(Clone, Message, PartialEq)]
pub struct Log {
    /// Index of the log which is similar to the subscript of the array
    #[prost(uint64, tag = "1")]
    pub index: u64,
    /// Log's term
    #[prost(uint64, tag = "2")]
    pub term: u64,
    /// Log's content
    #[prost(bytes, tag = "3")]
    pub log: Vec<u8>,
}

impl Log {
    // Constructs a new, empty `Log`.
    fn new() -> Self {
        Log {
            index: 0,
            term: 0,
            log: vec![],
        }
    }
    // Construct a Log based on parameters such as index, term, and log passed in.
    fn from_data(index: u64, term: u64, log: &Vec<u8>) -> Vec<u8> {
        let log = Log {
            index,
            term,
            log: log.clone(),
        };
        let mut data = vec![];
        let _ret = labcodec::encode(&log, &mut data).map_err(Error::Encode);
        data
    }
}

/// Save the information of Raft state
#[derive(Clone, Message, PartialEq)]
pub struct RaftState {
    /// Current term
    #[prost(uint64, tag = "1")]
    pub term: u64,
    /// The voting information of the Raft node
    #[prost(int64, tag = "2")]
    pub voted_for: i64,
    /// Log vector
    #[prost(bytes, repeated, tag = "3")]
    pub logs: Vec<Vec<u8>>,
}

/// A single Raft peer.
pub struct Raft {
    // RPC end points of all peers
    peers: Vec<RaftClient>,
    // Object to hold this peer's persisted state
    persister: Box<dyn Persister>,
    // this peer's index into peers[]
    me: usize,
    state: Arc<State>,
    // Your data here (2A, 2B, 2C).
    // Look at the paper's Figure 2 for a description of what
    // state a Raft server must maintain.

    // Voting information, which ensure that only one vote is awarded for the same term
    voted_for: Option<usize>,
    apply_ch: UnboundedSender<ApplyMsg>,
    // Log entry, Subscript starts from 1
    log: Vec<Vec<u8>>,
    // The largest index of committed log entries
    commit_index: u64,
    // The largest index of applied log entries
    last_applied: u64,
    // For each server, the index of the next log entry that needs to be sent to it
    next_index: Option<Vec<u64>>,
    // For each server, the highest index of the log that has been copied to it
    match_index: Option<Vec<u64>>,
}

impl Raft {
    /// the service or tester wants to create a Raft server. the ports
    /// of all the Raft servers (including this one) are in peers. this
    /// server's port is peers[me]. all the servers' peers arrays
    /// have the same order. persister is a place for this server to
    /// save its persistent state, and also initially holds the most
    /// recent saved state, if any. apply_ch is a channel on which the
    /// tester or service expects Raft to send ApplyMsg messages.
    /// This method must return quickly.
    pub fn new(
        peers: Vec<RaftClient>,
        me: usize,
        persister: Box<dyn Persister>,
        apply_ch: UnboundedSender<ApplyMsg>,
    ) -> Raft {
        let raft_state = persister.raft_state();

        // Your initialization code here (2A, 2B, 2C).
        let mut rf = Raft {
            peers,
            persister,
            me,
            state: Arc::new(State::new()),
            voted_for: None,
            apply_ch,
            log: vec![vec![]],
            commit_index: 0,
            last_applied: 0,
            next_index: None,
            match_index: None,
        };
        // initialize from state persisted before a crash
        rf.restore(&raft_state);
        rf
    }

    /// Get this peer's index into peers[]
    pub fn get_id(&self) -> usize {
        self.me
    }
    /// Get this peer's term
    pub fn term(&self) -> u64 {
        self.state.term()
    }
    /// Whether it is leader
    pub fn is_leader(&self) -> bool {
        self.state.is_leader()
    }
    /// Whether it is candidate
    pub fn is_candidate(&self) -> bool {
        if self.state.is_leader() || self.state.is_follower() {
            return false;
        }
        true
    }
    /// Whether it is follower
    pub fn is_follower(&self) -> bool {
        self.state.is_follower()
    }
    /// Set the raft's term to `term`
    pub fn set_term(&mut self, term: u64) {
        let is_leader = self.is_leader();
        let is_follower = self.is_follower();
        let current_term = self.term();
        if term > current_term {
            // When the period becomes larger, set `voted_for` None
            // to ensure that only one vote can be made for the same term.
            self.voted_for = None;
        }
        let new_state = State {
            term,
            is_leader,
            is_follower,
        };
        self.state = Arc::new(new_state);
        self.persist();
    }
    /// Set the identity of the Raft node
    pub fn set_role(&mut self, is_leader: bool, is_follower: bool) {
        let term = self.term();
        let new_state = State {
            term,
            is_leader,
            is_follower,
        };
        self.state = Arc::new(new_state);
        if is_leader {
            self.next_index = Some(vec![self.log.len() as u64; self.peers.len()]);
            self.match_index = Some(vec![0; self.peers.len()]);
        } else if is_follower {
            self.next_index = None;
            self.match_index = None;
        } else {
            self.next_index = None;
            self.match_index = None;
            self.set_vote_for(Some(self.me));
        }
    }
    /// Get peers[]
    pub fn get_peers(&self) -> Vec<RaftClient> {
        self.peers.clone()
    }
    /// Get the len of peers[]
    pub fn get_peers_len(&self) -> usize {
        self.peers.len()
    }
    /// Get `voted_for` in Raft structure
    pub fn get_voted_for(&self) -> Option<usize> {
        self.voted_for
    }
    /// Set `voted_for`
    pub fn set_vote_for(&mut self, id: Option<usize>) {
        self.voted_for = id;
        self.persist();
    }
    /// Get the index of the last log entry
    pub fn get_last_log_index(&self) -> usize {
        self.log.len() - 1
    }
    /// Get the term of the last log entry
    pub fn get_last_log_term(&self) -> u64 {
        let log = self.get_log(self.get_last_log_index() as u64).unwrap();
        log.term
    }
    /// Get log in the form of `Vec<u8>` at the index of `index`
    pub fn get_log_vec(&self, index: u64) -> Option<Vec<u8>> {
        if ((self.log.len() - 1) as u64) < index {
            None
        } else {
            Some(self.log[index as usize].clone())
        }
    }
    /// Get log in the form of `Log` at the index of `index`
    pub fn get_log(&self, index: u64) -> Option<Log> {
        if ((self.log.len() - 1) as u64) < index {
            None
        } else {
            match labcodec::decode(&self.log[index as usize]) {
                Ok(log) => {
                    let log: Log = log;
                    Some(log)
                }
                Err(_) => {
                    panic!("decode {} error!", index);
                }
            }
        }
    }
    /// Get `next_index` in the Raft structure
    pub fn get_next_index(&self) -> Option<Vec<u64>> {
        self.next_index.clone()
    }
    /// Get `commit_index` in the Raft structure
    pub fn get_commit_index(&self) -> u64 {
        self.commit_index
    }
    /// Delete the logs after `index`, but not including `index`
    pub fn pop_log(&mut self, index: u64) {
        if ((self.log.len() - 1) as u64) < index {
            return;
        }
        let _delete: Vec<Vec<u8>> = self.log.drain((index as usize + 1)..).collect();
        self.persist();
        for de in &_delete {
            let log: Log = match labcodec::decode(&de) {
                Ok(l) => l,
                Err(_) => continue,
            };
            let entry = log.log.clone();
            let mut _entr: Option<Entry> = None;
            match labcodec::decode(&entry) {
                Ok(en) => {
                    _entr = Some(en);
                }
                Err(e) => {}
            }
            my_debug!(
                "id:{} pop log:[{}:{}:{:?}]",
                self.me,
                log.index,
                log.term,
                _entr
            );
        }
    }
    /// Push a log entry
    pub fn push_log(&mut self, index: u64, term: u64, entry: &Vec<u8>) {
        if self.log.len() as u64 != index {
            my_debug!(
                "error:id:{} push index:{} error log:[{}:{}]",
                self.me,
                self.log.len(),
                index,
                term
            );
            return;
        }
        self.log.push(Log::from_data(index, term, entry));
        self.persist();
        let mut _entr: Option<Entry> = None;
        match labcodec::decode(&entry) {
            Ok(en) => {
                _entr = Some(en);
            }
            Err(e) => {}
        }
        my_debug!("id:{} push log:[{}:{}:{:?}]", self.me, index, term, _entr);
    }
    /// Push a log entry which is Created
    pub fn push_log_vec(&mut self, log: &Vec<u8>) {
        self.log.push(log.clone());
        self.persist();
        let log: Log = match labcodec::decode(log) {
            Ok(l) => l,
            Err(_) => return,
        };
        let entry = log.log.clone();
        let mut _entr: Option<Entry> = None;
        match labcodec::decode(&entry) {
            Ok(en) => {
                _entr = Some(en);
            }
            Err(e) => {}
        }
        my_debug!(
            "id:{} push_vec log:[{}:{}:{:?}]",
            self.me,
            log.index,
            log.term,
            _entr
        );
    }
    /// Handling the return value of append_entries (Only the leader is valid)
    pub fn handle_append_entries_reply(&mut self, id: usize, result: bool, needed_index: u64) {
        if !self.is_leader() || self.match_index == None || self.next_index == None {
            return;
        }
        my_debug!(
            "leader:{} handle_append_entries_reply id:{} result:{} next_index:{}",
            self.me,
            id,
            result,
            needed_index
        );
        let mut match_index = self.match_index.clone().unwrap();
        let mut next_index = self.next_index.clone().unwrap();
        let old_match = match_index[id];
        let old_next = next_index[id];
        my_debug!("id:{} before match_index:{:?}", self.me, match_index);
        my_debug!("id:{} before next_index:{:?}", self.me, next_index);
        if result && needed_index > next_index[id] {
            // Received success,and `needed_index` is higher
            match_index[id] = needed_index - 1;
            next_index[id] = needed_index;
        } else if !result && needed_index == next_index[id] {
            // When the received `needed_index` is equal to the index when it is sent, it can be processed.
            // Otherwise, it indicates that it has been processed before.
            if next_index[id] < (SEND_ENTRIES_MAX + 1) {
                next_index[id] = 1;
            } else {
                next_index[id] -= SEND_ENTRIES_MAX;
            }
        }
        self.next_index = Some(next_index.clone());
        self.match_index = Some(match_index.clone());
        my_debug!("id:{} after match_index:{:?}", self.me, match_index);
        my_debug!("id:{} after next_index:{:?}", self.me, next_index);
        my_debug!(
            "Node id:{} handle_append_entries_reply id:{} next_index:[{}->{}] match_index:[{}->{}]",
            self.me,
            id,
            old_next,
            next_index[id],
            old_match,
            match_index[id]
        );
        if old_match < match_index[id] {
            // Put itself to the maximum possible commit index
            match_index[self.me] = match_index[id] + 1;
            match_index.sort();
            // Take the smaller median
            let n_commit_index = match_index[(self.get_peers_len() - 1) / 2];
            self.set_commit_index(n_commit_index);
        }
    }
    /// Set `commit_index`
    pub fn set_commit_index(&mut self, new_commit_index: u64) {
        if new_commit_index <= self.commit_index {
            my_debug!(
                "error:id:{} set_commit_index fail:[{}-{}]",
                self.me,
                self.commit_index,
                new_commit_index
            );
            return;
        }
        let log = self.get_log(new_commit_index).unwrap();
        if log.term != self.term() {
            // The log which is not the current term can not be committed.
            my_debug!(
                "id:{} new_commit_index:{} log_term:{} != now_term:{}",
                self.me,
                new_commit_index,
                log.term,
                self.term()
            );
            return;
        }
        my_debug!(
            "id:{} set_commit_index:[{}->{}]",
            self.me,
            self.commit_index,
            new_commit_index
        );
        self.commit_index = new_commit_index;
        self.update_applied();
    }
    /// Update `last_applied` and apply the committed logs
    pub fn update_applied(&mut self) {
        if self.commit_index > self.last_applied {
            let last = self.last_applied;
            let high = self.commit_index;
            for i in last..high {
                self.last_applied += 1;
                let log = self.log[self.last_applied as usize].clone();
                let entry: Log = match labcodec::decode(&log) {
                    Ok(log) => log,
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                };
                let mesg = ApplyMsg {
                    command_valid: true,
                    command: entry.log.clone(),
                    command_index: self.last_applied,
                };
                let _ret = self.apply_ch.unbounded_send(mesg);
                let mut _entr: Option<Entry> = None;
                match labcodec::decode(&entry.log) {
                    Ok(en) => {
                        _entr = Some(en);
                    }
                    Err(e) => {}
                }
                my_debug!(
                    "id:{} apply_ch:[{}:{:?}]",
                    self.me,
                    self.last_applied,
                    _entr
                );
            }
        }
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    fn persist(&mut self) {
        // Your code here (2C).
        // Example:
        // labcodec::encode(&self.xxx, &mut data).unwrap();
        // labcodec::encode(&self.yyy, &mut data).unwrap();
        // self.persister.save_raft_state(data);
        let mut state_bytes = vec![];
        let mut voted_for: i64 = -1;
        if self.voted_for.is_some() {
            voted_for = self.voted_for.clone().unwrap() as i64;
        }
        let mut raft_state = RaftState {
            term: self.term(),
            voted_for: voted_for,
            logs: vec![],
        };
        for i in 1..self.log.len() {
            // The log at the index of 0 is an empty log
            let log = self.log[i].clone();
            raft_state.logs.push(log);
        }
        let _ret = labcodec::encode(&raft_state, &mut state_bytes).map_err(Error::Encode);
        self.persister.save_raft_state(state_bytes);
    }

    /// restore previously persisted state.
    fn restore(&mut self, data: &[u8]) {
        if data.is_empty() {
            // bootstrap without any state?
            return;
        }
        // Your code here (2C).
        // Example:
        // match labcodec::decode(data) {
        //     Ok(o) => {
        //         self.xxx = o.xxx;
        //         self.yyy = o.yyy;
        //     }
        //     Err(e) => {
        //         panic!("{:?}", e);
        //     }
        // }
        match labcodec::decode(data) {
            Ok(state) => {
                let state: RaftState = state;
                let new_state = State {
                    // follower
                    term: state.term,
                    is_leader: false,
                    is_follower: true,
                };
                self.state = Arc::new(new_state);
                if state.voted_for == -1 {
                    self.voted_for = None;
                } else {
                    self.voted_for = Some(state.voted_for as usize);
                }

                my_debug!(
                    "Restore id:{} log_len:{} term:{}",
                    self.me,
                    state.logs.len(),
                    state.term,
                );
                for i in 0..state.logs.len() {
                    let log = state.logs[i].clone();
                    self.log.push(log.clone());

                    // Print recovered logs for debugging
                    match labcodec::decode(&log) {
                        Ok(log) => {
                            let log: Log = log;
                            my_debug!("id:{} restore log :{:?}", self.me, log);
                        }
                        Err(e) => {
                            panic!("{:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    /// example code to send a RequestVote RPC to a server.
    /// server is the index of the target server in peers.
    /// expects RPC arguments in args.
    ///
    /// The labrpc package simulates a lossy network, in which servers
    /// may be unreachable, and in which requests and replies may be lost.
    /// This method sends a request and waits for a reply. If a reply arrives
    /// within a timeout interval, This method returns Ok(_); otherwise
    /// this method returns Err(_). Thus this method may not return for a while.
    /// An Err(_) return can be caused by a dead server, a live server that
    /// can't be reached, a lost request, or a lost reply.
    ///
    /// This method is guaranteed to return (perhaps after a delay) *except* if
    /// the handler function on the server side does not return.  Thus there
    /// is no need to implement your own timeouts around this method.
    ///
    /// look at the comments in ../labrpc/src/mod.rs for more details.
    fn send_request_vote(&self, server: usize, args: &RequestVoteArgs) -> Result<RequestVoteReply> {
        let peer = &self.peers[server];
        // Your code here if you want the rpc becomes async.
        // Example:
        // ```
        // let (tx, rx) = channel();
        // peer.spawn(
        //     peer.request_vote(&args)
        //         .map_err(Error::Rpc)
        //         .then(move |res| {
        //             tx.send(res);
        //             Ok(())
        //         }),
        // );
        // rx.wait() ...
        // ```
        peer.request_vote(&args).map_err(Error::Rpc).wait()
    }

    pub fn send_append_entries(
        &self,
        server: usize,
        args: &RequestEntriesArgs,
    ) -> Result<RequestEntriesReply> {
        let peer = &self.peers[server];
        /*let (tx, rx) = mpsc::channel();
        peer.spawn(
            peer.append_entries(&args)
                .map_err(Error::Rpc)
               .then(move |res| {
                    tx.send(res);
                    Ok(())
                }),
        );
        rx.wait();
        Err(Error::Rpc)*/
        peer.append_entries(&args).map_err(Error::Rpc).wait()
    }
    // Upper layer call, add log entry
    fn start<M>(&mut self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        // Your code here (2B).
        if self.is_leader() {
            let index = self.log.len() as u64;
            let term = self.term();
            let mut buf = vec![];
            labcodec::encode(command, &mut buf).map_err(Error::Encode)?;
            self.push_log(index, term, &buf);
            my_debug!("Start push_log index:{}, term:{}", index, term);
            Ok((index, term))
        } else {
            Err(Error::NotLeader)
        }
    }
}

// Choose concurrency paradigm.
//
// You can either drive the raft state machine by the rpc framework,
//
// ```rust
// struct Node { raft: Arc<Mutex<Raft>> }
// ```
//
// or spawn a new thread runs the raft state machine and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    // Your code here.
    // Similar to Raft.me, for debugging
    me: usize,
    // Save the status information of the Raft
    raft: Arc<Mutex<Raft>>,
    // Timeout thread, unique for follower
    timeout_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    // Channel sender communicating with the timeout thread
    timeout_sender: Arc<Mutex<Option<Sender<i32>>>>,
    // Heartbeat thread, unique fo leader
    heartbeat_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    // Channel sender communicating with heartbeat threads
    heartbeat_sender: Arc<Mutex<Option<Sender<i32>>>>,
    // On-off mark
    close_status: Arc<Mutex<bool>>,
}

impl Node {
    /// Create a new raft service.
    pub fn new(raft: Raft) -> Node {
        // Your code here.
        let mut node = Node {
            me: raft.get_id(),
            raft: Arc::new(Mutex::new(raft)),
            timeout_handle: Arc::new(Mutex::new(None)),
            timeout_sender: Arc::new(Mutex::new(None)),
            heartbeat_handle: Arc::new(Mutex::new(None)),
            heartbeat_sender: Arc::new(Mutex::new(None)),
            close_status: Arc::new(Mutex::new(false)),
        };
        my_debug!(
            "New Node:{} leader:{} follower:{}",
            node.get_id(),
            node.is_leader(),
            node.is_follower()
        );
        node.create_timeout_thread();
        node.create_append_entries_thread();
        node
    }
    /// Initially, create a timeout thread
    pub fn create_timeout_thread(&mut self) {
        let (tx, rx) = mpsc::channel();
        let node = self.clone();
        my_debug!("create_timeout_thread, ID:{}", self.get_id());
        let handle = thread::spawn(move || {
            loop {
                if *node.close_status.lock().unwrap() {
                    // whether to close
                    my_debug!("ID:{} timeout_thread close.", node.get_id());
                    break;
                }
                if node.is_leader() {
                    // If it is a leader, blocking the thread
                    my_debug!("ID:{} timeout_thread park!", node.get_id());
                    thread::park();
                }
                let rand_time = Duration::from_millis(
                    rand::thread_rng().gen_range(TIMEOUT_LOW_MIC, TIMEOUT_HIGH_MIC),
                );
                // Received within the scheduled time
                let re = rx.recv_timeout(rand_time);
                if let Ok(_re) = re {
                    my_debug!(
                        "follower:{}, RRRR Reset time_out, term:{}",
                        node.get_id(),
                        node.term()
                    );
                    continue;
                } else {
                    // timeout
                    // Again, determine if need to close
                    if *node.close_status.lock().unwrap() {
                        my_debug!("ID:{} timeout_thread close.", node.get_id());
                        break;
                    }
                    if !node.is_leader() {
                        Node::start_vote(node.clone());
                    }
                }
            }
        });
        *self.timeout_handle.lock().unwrap() = Some(handle);
        *self.timeout_sender.lock().unwrap() = Some(tx);
    }
    /// Start voting
    pub fn start_vote(node: Node) {
        my_debug!("Node:{} start vote...", node.me);
        let mut raft = node.raft.lock().unwrap();
        my_debug!("Node:{} Lock!", node.me);
        if raft.is_leader() {
            return;
        }
        let term = raft.term() + 1;
        // inc term
        raft.set_term(term);
        // become candidate
        raft.set_role(false, false);
        let id = raft.get_id();
        let args = RequestVoteArgs {
            term,
            candidate_id: id as u64,
            last_log_index: raft.get_last_log_index() as u64,
            last_log_term: raft.get_last_log_term(),
        };
        my_debug!(
            "ID:{} term:{} args:{:?} start vote...",
            raft.get_id(),
            raft.term(),
            args
        );
        // Give itself a vote first
        let poll = Arc::new(Mutex::new(1));
        let len = raft.get_peers_len();
        let peers = raft.get_peers();
        for i in 0..len {
            if i == id {
                continue;
            }
            let time = Instant::now();
            let me = id;
            let len = len as u64;
            let node = node.clone();
            let poll = Arc::clone(&poll);
            let args = args.clone();
            let peer = peers[i].clone();
            let term = term;
            peer.spawn(
                peer.request_vote(&args)
                    .map(move |ret| {
                        let mut raft = node.raft.lock().unwrap();
                        if ret.term > raft.term() {
                            // Encounter a node whose term is larger
                            // Firstly, reset the timeout thread timing to avoid timeout
                            let _re = node.timeout_sender.lock().unwrap().clone().unwrap().send(1);
                            raft.set_term(ret.term);
                            // become follower
                            raft.set_role(false, true);
                            let time2 = time.elapsed().as_millis();
                            my_debug!(
                                "candidate:{} get vote_granted:{} false, higher term:{} time:{}!",
                                me,
                                i,
                                ret.term,
                                time2
                            );
                        } else {
                            // If the current node is not a candidate,
                            // or if the term changes, the vote is invalid
                            if !raft.is_candidate() || term != raft.term() {
                                return;
                            } else {
                                if ret.vote_granted {
                                    *poll.lock().unwrap() += 1;
                                    let time2 = time.elapsed().as_millis();
                                    my_debug!(
                                        "candidate:{} get vote_granted:{} true time:{}!",
                                        me,
                                        i,
                                        time2
                                    );
                                    // More than half of the votes
                                    if *poll.lock().unwrap() > len / 2 {
                                        // become leader
                                        // Reset send log thread timing, send heartbeat immediately
                                        let _re = node
                                            .heartbeat_sender
                                            .lock()
                                            .unwrap()
                                            .clone()
                                            .unwrap()
                                            .send(1);
                                        raft.set_role(true, false);
                                        node.leader_exchange_follower(false);
                                        my_debug!(
                                            "candidate:{} become leader! grantesd from node:{} poll:{}",
                                            me,
                                            i,
                                            *poll.lock().unwrap()
                                        );
                                    }
                                }
                            }
                        }
                    })
                    .map_err(|_| {}),
            );
        }
    }
    /// Initially, create a append-entries thread
    pub fn create_append_entries_thread(&mut self) {
        let (tx, rx) = mpsc::channel();
        let node = self.clone();
        my_debug!("create_append_entries_thread, ID:{}", node.get_id());
        let handle = thread::spawn(move || {
            loop {
                if *node.close_status.lock().unwrap() {
                    // Whether to close
                    my_debug!("ID:{} heartbeat_thread close.", node.get_id());
                    break;
                }
                if !node.is_leader() {
                    // If it is not a leader, block thread
                    thread::park();
                } else {
                    Node::send_entries(node.clone());
                }
                let time = Duration::from_millis(HEARTBEAT_INTERVAL);
                let re = rx.recv_timeout(time);
                if re.is_ok() {
                    // Received a reset signal
                    continue;
                }
            }
        });
        *self.heartbeat_handle.lock().unwrap() = Some(handle);
        *self.heartbeat_sender.lock().unwrap() = Some(tx);
    }
    /// Send log/heartbeat
    pub fn send_entries(node: Node) {
        // Lock to prevent term changes
        let raft = node.raft.lock().unwrap();
        let id = raft.get_id();
        let next_index = match raft.get_next_index() {
            Some(index) => index.clone(),
            None => {
                return;
            }
        };
        my_debug!(
            "Leader:{} term:{} send entries log_len:{} next_index:{:?}",
            id,
            raft.term(),
            raft.get_last_log_index() + 1,
            next_index
        );
        let len = raft.get_peers_len();
        let peers = raft.get_peers();

        for i in 0..len {
            if i == id {
                continue;
            }
            let time = Instant::now();
            let me = id;
            let len = len as u64;
            let node = node.clone();
            let peer = peers[i].clone();
            let prev_log_index = next_index[i] - 1;
            let mut args = RequestEntriesArgs {
                term: raft.term(),
                leader_id: id as u64,
                prev_log_index: next_index[i] - 1,
                prev_log_term: 0,
                entries: vec![],
                leader_commit: raft.get_commit_index(),
            };
            let entries = raft.get_log(prev_log_index);
            if entries.is_none() {
                my_debug!(
                    "Error:leader:{} prev_log_index:{} no log, log len:{}",
                    id,
                    prev_log_index,
                    raft.get_last_log_index() + 1
                );
                return;
            }
            args.prev_log_term = entries.unwrap().term;
            for j in 0..SEND_ENTRIES_MAX {
                // Send up to SEND_ENTRIES_MAX logs at a time
                let entry_next = raft.get_log_vec(next_index[i] + j);
                match entry_next {
                    Some(en) => {
                        args.entries.push(en);
                    }
                    None => {
                        break;
                    }
                }
            }

            my_debug!("leader[id:{}, term:{}] send entries id:{} args:[term:{} prev_index:{} prev_term:{} commit:{} entry_num:{}]",
            me, raft.term(), i, args.term, args.prev_log_index, args.prev_log_term, args.leader_commit, args.entries.len());

            peer.spawn(
                peer.append_entries(&args).map(move |ret| {
                    let mut raft = node.raft.lock().unwrap();
                    if ret.term > raft.term() {
                        // First reset the timeout thread timing to avoid timeout
                        let _re = node.timeout_sender.lock().unwrap().clone().unwrap().send(1);
                        raft.set_term(ret.term);
                        raft.set_role(false, true);
                        // Release timeout thread blocking
                        node.leader_exchange_follower(true);
                        let time2 = time.elapsed().as_millis();
                        my_debug!(
                            "leader:[id:{} term:{}] get logs_granted:{} false, higher term:{} time:{}!!!",
                            me,
                            raft.term(),
                            i,
                            ret.term,
                            time2
                        );
                    } else if ret.term == raft.term() {
                        // deal if the term is not changed
                        if ret.entries_granted {
                            raft.handle_append_entries_reply(i, true, ret.next_index);
                            let time2 = time.elapsed().as_millis();
                            my_debug!("leader:{} get logs_granted:{} true time:{}!", me, i, time2);
                        } else {
                            raft.handle_append_entries_reply(i, false, ret.next_index);
                            let time2 = time.elapsed().as_millis();
                            my_debug!(
                                "leader:{} get logs_granted from node:{} false, needed next_index:{} time:{}!",
                                me,
                                i,
                                ret.next_index,
                                time2
                            );
                        }
                    }
                }).map_err(|_|{})
            );
        }
    }
    /// Unpark heartbeat thread
    pub fn heartbeat_thread_unpark(&self) {
        if let Some(ref handle) = *self.heartbeat_handle.lock().unwrap() {
            handle.thread().unpark();
        }
    }
    /// Unpark timeout thread
    pub fn timeout_thread_unpark(&self) {
        if let Some(ref handle) = *self.timeout_handle.lock().unwrap() {
            handle.thread().unpark();
        }
    }
    /// According to mode, Unpark heartbeat thread or timeout thread
    pub fn leader_exchange_follower(&self, mode: bool) {
        if mode {
            self.timeout_thread_unpark();
        } else {
            self.heartbeat_thread_unpark();
        }
    }

    /// the service using Raft (e.g. a k/v server) wants to start
    /// agreement on the next command to be appended to Raft's log. if this
    /// server isn't the leader, returns [`Error::NotLeader`]. otherwise start
    /// the agreement and return immediately. there is no guarantee that this
    /// command will ever be committed to the Raft log, since the leader
    /// may fail or lose an election. even if the Raft instance has been killed,
    /// this function should return gracefully.
    ///
    /// the first value of the tuple is the index that the command will appear
    /// at if it's ever committed. the second is the current term.
    ///
    /// This method must return without blocking on the raft.
    pub fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        // Your code here.
        if self.is_leader() {
            my_debug!("leader:{}", self.get_id());
            self.raft.lock().unwrap().start(command)
        } else {
            Err(Error::NotLeader)
        }
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        // Your code here.
        // Example:
        // self.raft.term
        self.raft.lock().unwrap().term()
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        // Your code here.
        // Example:
        // self.raft.leader_id == self.id
        self.raft.lock().unwrap().is_leader()
    }

    /// Whether this peer believes it is the follower.
    pub fn is_follower(&self) -> bool {
        self.raft.lock().unwrap().is_follower()
    }

    /// The current state of this peer.
    pub fn get_state(&self) -> State {
        State {
            term: self.term(),
            is_leader: self.is_leader(),
            is_follower: self.is_follower(),
        }
    }
    /// Get this peer's index into peers[]
    pub fn get_id(&self) -> usize {
        self.me
    }

    /// the tester calls kill() when a Raft instance won't be
    /// needed again. you are not required to do anything in
    /// kill(), but it might be convenient to (for example)
    /// turn off debug output from this instance.
    /// In Raft paper, a server crash is a PHYSICAL crash,
    /// A.K.A all resources are reset. But we are simulating
    /// a VIRTUAL crash in tester, so take care of background
    /// threads you generated with this Raft Node.
    pub fn kill(&self) {
        // Your code here, if desired.
        *self.close_status.lock().unwrap() = true;
        match *self.timeout_sender.lock().unwrap() {
            Some(ref send) => {
                let _ret = send.send(1);
            }
            None => {}
        }
        match *self.heartbeat_sender.lock().unwrap() {
            Some(ref send) => {
                let _ret = send.send(1);
            }
            None => {}
        }
        self.timeout_thread_unpark();
        self.heartbeat_thread_unpark();
        let timeout_thread = self.timeout_handle.lock().unwrap().take();
        if timeout_thread.is_some() {
            let _ = timeout_thread.unwrap().join();
        }
        let heartbeat_thread = self.heartbeat_handle.lock().unwrap().take();
        if heartbeat_thread.is_some() {
            let _ = heartbeat_thread.unwrap().join();
        }
    }
}

impl RaftService for Node {
    // example RequestVote RPC handler.
    fn request_vote(&self, args: RequestVoteArgs) -> RpcFuture<RequestVoteReply> {
        // Your code here (2A, 2B).
        let mut raft = self.raft.lock().unwrap();
        let mut reply = RequestVoteReply {
            term: raft.term(),
            vote_granted: false,
        };
        let current_term = raft.term();
        let term = args.term;
        if current_term > term {
            reply.vote_granted = false;
        } else {
            if current_term < term {
                // term is smaller, becomes follower
                if raft.is_leader() {
                    my_debug!(
                        "LLEADER:{} Become follower. because vote from node:{}",
                        raft.get_id(),
                        args.candidate_id
                    );
                }
                if raft.is_candidate() {
                    my_debug!(
                        "CCANDIDATE:{} Become follower. because vote from Candidate:{}",
                        raft.get_id(),
                        args.candidate_id
                    );
                }
                raft.set_term(term);
                // Become a follower (while setting voted_for None)
                raft.set_role(false, true);
                self.leader_exchange_follower(true);
            }
            if ((args.last_log_term == raft.get_last_log_term()
                && args.last_log_index >= raft.get_last_log_index() as u64)
                || args.last_log_term > raft.get_last_log_term() as u64)
                && raft.voted_for.is_none()
            {
                // Not voting
                let _re = self.timeout_sender.lock().unwrap().clone().unwrap().send(1);
                reply.vote_granted = true;
                raft.set_vote_for(Some(args.candidate_id as usize))
            }
        }
        my_debug!(
            "[ID:{}, term:{}] Receive vote [ID:{}, term:{}]  vote {}",
            raft.get_id(),
            raft.term(),
            args.candidate_id,
            args.term,
            reply.vote_granted
        );
        Box::new(futures::future::result(Ok(reply)))
    }

    fn append_entries(&self, args: RequestEntriesArgs) -> RpcFuture<RequestEntriesReply> {
        my_debug!(
            "FFFF ID:{} Receive entries from id:{} pre_term:{}, pre_index:{}......",
            self.me,
            args.leader_id,
            args.prev_log_term,
            args.prev_log_index
        );
        let mut raft = self.raft.lock().unwrap();
        my_debug!("Lock！id:{}", self.me);
        let mut reply = RequestEntriesReply {
            term: raft.term(),
            entries_granted: false,
            // When returning false, it can be used as a judgment
            next_index: args.prev_log_index + 1,
        };
        let current_term = raft.term();
        if args.term < current_term {
            my_debug!(
                "Follower[id:{},term:{}] receive entries Leader[id:{},term:{}] false",
                raft.get_id(),
                raft.term(),
                args.leader_id,
                args.term
            );
        } else {
            let _re = self.timeout_sender.lock().unwrap().clone().unwrap().send(1); //先发送心跳，避免超时
            let entries = raft.get_log(args.prev_log_index);
            match entries {
                Some(en) => {
                    // entry exist
                    if en.term == args.prev_log_term {
                        // The term is the same, the match is successful
                        if args.entries.len() == 0 {
                            // heartbeat
                            my_debug!(
                                "Follower[id:{},term:{}] receive heartbeat Leader[id:{},term:{}]",
                                raft.get_id(),
                                raft.term(),
                                args.leader_id,
                                args.term
                            );
                        } else {
                            // log entries
                            my_debug!("Follower[id:{},term:{}] receive entries len:{} Leader[id:{},term:{}]", raft.get_id(), raft.term(), args.entries.len(), args.leader_id, args.term);
                            for i in 0..args.entries.len() {
                                let log = args.entries[i].clone();
                                let node_entry =
                                    raft.get_log_vec(args.prev_log_index + 1 + i as u64);
                                match node_entry {
                                    Some(raft_log) => {
                                        if raft_log == log {
                                            // both logs are the same, do not delete
                                            continue;
                                        } else {
                                            raft.pop_log(args.prev_log_index + i as u64);
                                            raft.push_log_vec(&log);
                                        }
                                    }
                                    None => {
                                        raft.push_log_vec(&log);
                                    }
                                }
                            }
                        }
                        // match successfully, return true
                        reply.entries_granted = true;
                        reply.next_index = args.prev_log_index + 1 + args.entries.len() as u64;
                        my_debug!("ff id:{} term:{} commit_index:{} log_len:{} receive args:[pre_term:{} pre_index:{} log_len:{} commit:{}]", 
                            raft.get_id(), raft.term(), raft.get_commit_index(), raft.get_last_log_index() + 1,
                            args.prev_log_term, args.prev_log_index, args.entries.len(), args.leader_commit);
                        // take the Minimum value
                        let new_commit_index: u64 =
                            cmp::min(args.leader_commit, raft.get_last_log_index() as u64);
                        raft.set_commit_index(new_commit_index);
                        // Reset the timing again to avoid timeouts
                        let _re = self.timeout_sender.lock().unwrap().clone().unwrap().send(1);
                    } else {
                        // Match failed, wait until the match is successful and then delete logs
                        my_debug!(
                            "Follower[{}:{}-{}:{}] match fail leader[{}:{}-{}:{}]",
                            raft.get_id(),
                            raft.term(),
                            en.term,
                            raft.get_last_log_index(),
                            args.leader_id,
                            args.term,
                            args.prev_log_term,
                            args.prev_log_index
                        );
                    }
                }
                None => {} // Match failed
            }
            if raft.is_leader() {
                my_debug!(
                    "LLEADER:{} Become follower. because entries from Leader:{}",
                    raft.get_id(),
                    args.leader_id
                );
            }
            if raft.is_candidate() {
                my_debug!(
                    "CCANDIDATE:{} Become follower. because entries from Leader:{}",
                    raft.get_id(),
                    args.leader_id
                );
            }
            if args.term > raft.term() || args.term == raft.term() && raft.is_candidate() {
                raft.set_term(args.term);
                raft.set_role(false, true);
                raft.set_vote_for(Some(args.leader_id as usize));
                self.leader_exchange_follower(true);
            }
        }
        reply.term = raft.term();
        Box::new(futures::future::result(Ok(reply)))
    }
}
