extern crate futures;
extern crate grpcio;
extern crate protobuf;

use super::database;
use super::protos;

use database::{Key, Value};
use grpcio::{ChannelBuilder, EnvBuilder};
use protos::kvserver::{
    ClearRequest, ClearResponse, DeleteRequest, DeleteResponse, GetRequest, GetResponse,
    InsertRequest, InsertResponse, ScanAllRequest, ScanAllResponse, ScanRequest, ScanResponse,
    SetRequest, SetResponse, UpdateRequest, UpdateResponse,
};
use protos::kvserver_grpc::KvdbClient;
use std::sync::Arc;

/// Client backend operation
///
/// Provide a series of operations on the database
/// Each operation will communicate with the server via rpc
pub struct Client {
    client: KvdbClient,
}

impl Client {
    /// Constructs a new ClientOP
    ///
    /// Bind the incoming IP address and port number
    pub fn new(host: String, port: u16) -> Self {
        let addr = format!("{}:{}", host, port);
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(addr.as_ref());
        let kv_client = KvdbClient::new(ch);

        Client { client: kv_client }
    }

    /// get operation
    pub fn get(&self, key: Key) -> GetResponse {
        let mut request = GetRequest::new();
        request.set_key(key);
        self.client.get(&request).expect("RPC failed")
    }

    /// set operation
    pub fn set(&self, key: Key, value: Value) -> SetResponse {
        let mut request = SetRequest::new();
        request.set_key(key);
        request.set_value(value);
        self.client.set(&request).expect("RPC failed")
    }

    /// insert operation
    pub fn insert(&self, key: Key, value: Value) -> InsertResponse {
        let mut request = InsertRequest::new();
        request.set_key(key);
        request.set_value(value);
        self.client.insert(&request).expect("RPC failed")
    }

    /// update operation
    pub fn update(&self, key: Key, value: Value) -> UpdateResponse {
        let mut request = UpdateRequest::new();
        request.set_key(key);
        request.set_value(value);
        self.client.update(&request).expect("RPC failed")
    }

    /// delete operation
    pub fn delete(&self, key: String) -> DeleteResponse {
        let mut request = DeleteRequest::new();
        request.set_key(key);
        self.client.delete(&request).expect("RPC failed")
    }

    /// scan operation
    pub fn scan(&self, key_start: String, key_end: String) -> ScanResponse {
        let mut request = ScanRequest::new();
        request.set_key_start(key_start);
        request.set_key_end(key_end);
        self.client.scan(&request).expect("RPC failed")
    }

    /// scan_all operation
    pub fn scan_all(&self) -> ScanAllResponse {
        let request = ScanAllRequest::new();
        self.client.scan_all(&request).expect("RPC failed")
    }

    /// clear operation
    pub fn clear(&self) -> ClearResponse {
        let request = ClearRequest::new();
        self.client.clear(&request).expect("RPC failed")
    }
}
