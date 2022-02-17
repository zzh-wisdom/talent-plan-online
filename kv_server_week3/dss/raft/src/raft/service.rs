labrpc::service! {
    service raft {
        rpc request_vote(RequestVoteArgs) returns (RequestVoteReply);
        rpc append_entries(RequestEntriesArgs) returns (RequestEntriesReply);
        // Your code here if more rpc desired.
        // rpc xxx(yyy) returns (zzz)
    }
}
pub use self::raft::{
    add_service as add_raft_service, Client as RaftClient, Service as RaftService,
};

/// Example RequestVote RPC arguments structure.
#[derive(Clone, PartialEq, Message)]
pub struct RequestVoteArgs {
    // Your data here (2A, 2B).
    // Candidate's term number
    #[prost(uint64, tag = "1")]
    pub term: u64,
    // The id of the candidate who requested the voting
    #[prost(uint64, tag = "2")]
    pub candidate_id: u64,
    // Index of the candidate's last log entry
    #[prost(uint64, tag = "3")]
    pub last_log_index: u64,
    // The term of the candidate's last log entry
    #[prost(uint64, tag = "4")]
    pub last_log_term: u64,
}

// Example RequestVote RPC reply structure.
#[derive(Clone, PartialEq, Message)]
pub struct RequestVoteReply {
    // Your data here (2A).
    // Current term number which is convenient for candidate to update its tenure number
    #[prost(uint64, tag = "1")]
    pub term: u64,
    // `vote_granted` is true when the candidate wins this vote
    #[prost(bool, tag = "2")]
    pub vote_granted: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct RequestEntriesArgs {
    // Your data here (2A, 2B).
    // leader's term
    #[prost(uint64, tag = "1")]
    pub term: u64,
    // leader's id
    #[prost(uint64, tag = "2")]
    pub leader_id: u64,

    #[prost(uint64, tag = "3")]
    pub prev_log_index: u64,

    #[prost(uint64, tag = "4")]
    pub prev_log_term: u64,

    #[prost(bytes, repeated, tag = "5")]
    pub entries: Vec<Vec<u8>>,

    #[prost(uint64, tag = "6")]
    pub leader_commit: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct RequestEntriesReply {
    // Your data here (2A).
    #[prost(uint64, tag = "1")]
    pub term: u64,

    #[prost(bool, tag = "2")]
    pub entries_granted: bool,
    // The next index it want to receive
    #[prost(uint64, tag = "3")]
    pub next_index: u64,
}
