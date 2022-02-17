extern crate futures;
extern crate grpcio;
extern crate protobuf;

use super::database;
use super::protos;

use futures::Future;
use grpcio::{Environment, RpcContext, Server, ServerBuilder, UnarySink};
use std::sync::Arc;

use database::btreemap_engine::BtreeMapEngine;
use database::engine_trait::Engine;
use database::Database;
use protos::kvserver::{
    ClearRequest, ClearResponse, DeleteRequest, DeleteResponse, GetRequest, GetResponse,
    InsertRequest, InsertResponse, ResponseStatus, ScanAllRequest, ScanAllResponse, ScanRequest,
    ScanResponse, SetRequest, SetResponse, UpdateRequest, UpdateResponse,
};
use protos::kvserver_grpc::{self, Kvdb};

/// Server service
#[derive(Clone)]
struct DbService {
    database: Database<BtreeMapEngine>,
}

impl DbService {
    /// Constructs a new `DbService`
    pub fn new() -> Self {
        DbService {
            database: Database::new(),
        }
    }

    /// Close DbService
    pub fn stop(&mut self) {
        println!("DbService stop...");
        self.database.stop();
    }
}

/// Various operations provided by the database
impl Kvdb for DbService {
    fn get(&mut self, ctx: RpcContext, req: GetRequest, sink: UnarySink<GetResponse>) {
        let mut response = GetResponse::new();
        println!("GET:Received GetRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        let ret = engine.get(req.key);
        match ret {
            Ok(op) => match op {
                Some(value) => {
                    response.set_status(ResponseStatus::kSuccess);
                    response.set_value(value);
                }
                None => response.set_status(ResponseStatus::kNotFound),
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("GET:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("GET:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn set(&mut self, ctx: RpcContext, req: SetRequest, sink: UnarySink<SetResponse>) {
        let mut response = SetResponse::new();
        let engine = &mut self.database.engine;
        let ret = engine.set(req.key, req.value);
        match ret {
            Ok(op) => match op {
                Some(value) => {
                    response.set_status(ResponseStatus::kSuccess);
                    response.set_old_value(value);
                    response.set_empty(false);
                }
                None => {
                    response.set_status(ResponseStatus::kSuccess);
                    response.set_empty(true);
                }
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            //.map(move |_| println!("SET:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("SET:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn insert(&mut self, ctx: RpcContext, req: InsertRequest, sink: UnarySink<InsertResponse>) {
        let mut response = InsertResponse::new();
        println!("INSERT:Received InsertRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        let ret = engine.insert(req.key, req.value);
        match ret {
            Ok(op) => match op {
                true => response.set_status(ResponseStatus::kSuccess),
                false => response.set_status(ResponseStatus::kFailed),
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("INSERT:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("INSERT:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn update(&mut self, ctx: RpcContext, req: UpdateRequest, sink: UnarySink<UpdateResponse>) {
        let mut response = UpdateResponse::new();
        println!("UPDATE:Received UpdateRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        let ret = engine.update(req.key, req.value);
        match ret {
            Ok(op) => match op {
                Some(value) => {
                    response.set_status(ResponseStatus::kSuccess);
                    response.set_old_value(value);
                }
                None => response.set_status(ResponseStatus::kFailed),
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("UPDATE:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("UPDATE:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn delete(&mut self, ctx: RpcContext, req: DeleteRequest, sink: UnarySink<DeleteResponse>) {
        let mut response = DeleteResponse::new();
        println!("DELET:Received DeleteRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        let ret = engine.delete(req.key);
        match ret {
            Ok(op) => match op {
                Some(value) => {
                    response.set_status(ResponseStatus::kSuccess);
                    response.set_delete_value(value);
                }
                None => response.set_status(ResponseStatus::kNotFound),
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("DELET:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("DELET:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn scan(&mut self, ctx: RpcContext, req: ScanRequest, sink: UnarySink<ScanResponse>) {
        let mut response = ScanResponse::new();
        println!("SCAN:Received ScanRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        // Make sure key_end is greater than key_start
        if req.key_start >= req.key_end {
            response.set_status(ResponseStatus::kNotFound);
        } else {
            let ret = engine.scan(req.key_start, req.key_end);
            match ret {
                Ok(op) => match op {
                    Some(key_value) => {
                        response.set_status(ResponseStatus::kSuccess);
                        response.set_key_value(key_value);
                    }
                    None => response.set_status(ResponseStatus::kNotFound),
                },
                Err(_) => response.set_status(ResponseStatus::kError),
            }
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("SCAN:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("SCAN:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn scan_all(&mut self, ctx: RpcContext, req: ScanAllRequest, sink: UnarySink<ScanAllResponse>) {
        let mut response = ScanAllResponse::new();
        println!("SCANALL:Received ScanAllRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        let ret = engine.scan_all();
        match ret {
            Ok(op) => match op {
                Some(key_value) => {
                    response.set_status(ResponseStatus::kSuccess);
                    response.set_key_value(key_value);
                }
                None => response.set_status(ResponseStatus::kNotFound),
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("SCANALL:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("SCANALL:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
    fn clear(&mut self, ctx: RpcContext, req: ClearRequest, sink: UnarySink<ClearResponse>) {
        let mut response = ClearResponse::new();
        println!("CLEAR:Received ClearRequest {{ {:?} }}", req);
        let engine = &mut self.database.engine;
        let ret = engine.clear();
        match ret {
            Ok(op) => match op {
                true => response.set_status(ResponseStatus::kSuccess),
                false => response.set_status(ResponseStatus::kFailed),
            },
            Err(_) => response.set_status(ResponseStatus::kError),
        }
        let f = sink
            .success(response.clone())
            .map(move |_| println!("CLEAR:Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("CLEAR:Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
}

/// The structure that actually interacts with the client
pub struct KvServer {
    grpc_server: Server,
    dbservice: DbService,
}

impl KvServer {
    /// Constructs a new `DbService`
    ///
    /// Listen for the IP addresses and port numbers
    pub fn new(host: String, port: u16) -> Self {
        let env = Arc::new(Environment::new(4));
        let dbservice = DbService::new();
        let service = kvserver_grpc::create_kvdb(dbservice.clone());
        let grpc_server = ServerBuilder::new(env)
            .register_service(service)
            .bind(host, port.clone())
            .build()
            .unwrap();
        KvServer {
            grpc_server,
            dbservice,
        }
    }

    /// Open the database and start the service
    pub fn start(&mut self) {
        self.grpc_server.start();
        for &(ref host, port) in self.grpc_server.bind_addrs() {
            println!("listening on {}:{}", host, port);
        }
    }

    /// Close server
    ///
    /// Need to wait for the end of all the underlying threads
    pub fn stop(&mut self) {
        println!("Server stop...");
        self.dbservice.stop();
        let _ = self.grpc_server.shutdown().wait();
    }
}
