// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_KVDB_GET: ::grpcio::Method<super::kvserver::GetRequest, super::kvserver::GetResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Get",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_SET: ::grpcio::Method<super::kvserver::SetRequest, super::kvserver::SetResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Set",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_INSERT: ::grpcio::Method<super::kvserver::InsertRequest, super::kvserver::InsertResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Insert",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_UPDATE: ::grpcio::Method<super::kvserver::UpdateRequest, super::kvserver::UpdateResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Update",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_DELETE: ::grpcio::Method<super::kvserver::DeleteRequest, super::kvserver::DeleteResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Delete",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_SCAN: ::grpcio::Method<super::kvserver::ScanRequest, super::kvserver::ScanResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Scan",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_SCAN_ALL: ::grpcio::Method<super::kvserver::ScanAllRequest, super::kvserver::ScanAllResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/ScanAll",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KVDB_CLEAR: ::grpcio::Method<super::kvserver::ClearRequest, super::kvserver::ClearResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvserver.Kvdb/Clear",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct KvdbClient {
    client: ::grpcio::Client,
}

impl KvdbClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        KvdbClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_opt(&self, req: &super::kvserver::GetRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::GetResponse> {
        self.client.unary_call(&METHOD_KVDB_GET, req, opt)
    }

    pub fn get(&self, req: &super::kvserver::GetRequest) -> ::grpcio::Result<super::kvserver::GetResponse> {
        self.get_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_async_opt(&self, req: &super::kvserver::GetRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::GetResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_GET, req, opt)
    }

    pub fn get_async(&self, req: &super::kvserver::GetRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::GetResponse>> {
        self.get_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn set_opt(&self, req: &super::kvserver::SetRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::SetResponse> {
        self.client.unary_call(&METHOD_KVDB_SET, req, opt)
    }

    pub fn set(&self, req: &super::kvserver::SetRequest) -> ::grpcio::Result<super::kvserver::SetResponse> {
        self.set_opt(req, ::grpcio::CallOption::default())
    }

    pub fn set_async_opt(&self, req: &super::kvserver::SetRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::SetResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_SET, req, opt)
    }

    pub fn set_async(&self, req: &super::kvserver::SetRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::SetResponse>> {
        self.set_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn insert_opt(&self, req: &super::kvserver::InsertRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::InsertResponse> {
        self.client.unary_call(&METHOD_KVDB_INSERT, req, opt)
    }

    pub fn insert(&self, req: &super::kvserver::InsertRequest) -> ::grpcio::Result<super::kvserver::InsertResponse> {
        self.insert_opt(req, ::grpcio::CallOption::default())
    }

    pub fn insert_async_opt(&self, req: &super::kvserver::InsertRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::InsertResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_INSERT, req, opt)
    }

    pub fn insert_async(&self, req: &super::kvserver::InsertRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::InsertResponse>> {
        self.insert_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_opt(&self, req: &super::kvserver::UpdateRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::UpdateResponse> {
        self.client.unary_call(&METHOD_KVDB_UPDATE, req, opt)
    }

    pub fn update(&self, req: &super::kvserver::UpdateRequest) -> ::grpcio::Result<super::kvserver::UpdateResponse> {
        self.update_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_async_opt(&self, req: &super::kvserver::UpdateRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::UpdateResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_UPDATE, req, opt)
    }

    pub fn update_async(&self, req: &super::kvserver::UpdateRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::UpdateResponse>> {
        self.update_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_opt(&self, req: &super::kvserver::DeleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::DeleteResponse> {
        self.client.unary_call(&METHOD_KVDB_DELETE, req, opt)
    }

    pub fn delete(&self, req: &super::kvserver::DeleteRequest) -> ::grpcio::Result<super::kvserver::DeleteResponse> {
        self.delete_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_async_opt(&self, req: &super::kvserver::DeleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::DeleteResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_DELETE, req, opt)
    }

    pub fn delete_async(&self, req: &super::kvserver::DeleteRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::DeleteResponse>> {
        self.delete_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn scan_opt(&self, req: &super::kvserver::ScanRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::ScanResponse> {
        self.client.unary_call(&METHOD_KVDB_SCAN, req, opt)
    }

    pub fn scan(&self, req: &super::kvserver::ScanRequest) -> ::grpcio::Result<super::kvserver::ScanResponse> {
        self.scan_opt(req, ::grpcio::CallOption::default())
    }

    pub fn scan_async_opt(&self, req: &super::kvserver::ScanRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::ScanResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_SCAN, req, opt)
    }

    pub fn scan_async(&self, req: &super::kvserver::ScanRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::ScanResponse>> {
        self.scan_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn scan_all_opt(&self, req: &super::kvserver::ScanAllRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::ScanAllResponse> {
        self.client.unary_call(&METHOD_KVDB_SCAN_ALL, req, opt)
    }

    pub fn scan_all(&self, req: &super::kvserver::ScanAllRequest) -> ::grpcio::Result<super::kvserver::ScanAllResponse> {
        self.scan_all_opt(req, ::grpcio::CallOption::default())
    }

    pub fn scan_all_async_opt(&self, req: &super::kvserver::ScanAllRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::ScanAllResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_SCAN_ALL, req, opt)
    }

    pub fn scan_all_async(&self, req: &super::kvserver::ScanAllRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::ScanAllResponse>> {
        self.scan_all_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn clear_opt(&self, req: &super::kvserver::ClearRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvserver::ClearResponse> {
        self.client.unary_call(&METHOD_KVDB_CLEAR, req, opt)
    }

    pub fn clear(&self, req: &super::kvserver::ClearRequest) -> ::grpcio::Result<super::kvserver::ClearResponse> {
        self.clear_opt(req, ::grpcio::CallOption::default())
    }

    pub fn clear_async_opt(&self, req: &super::kvserver::ClearRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::ClearResponse>> {
        self.client.unary_call_async(&METHOD_KVDB_CLEAR, req, opt)
    }

    pub fn clear_async(&self, req: &super::kvserver::ClearRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvserver::ClearResponse>> {
        self.clear_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Kvdb {
    fn get(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::GetRequest, sink: ::grpcio::UnarySink<super::kvserver::GetResponse>);
    fn set(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::SetRequest, sink: ::grpcio::UnarySink<super::kvserver::SetResponse>);
    fn insert(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::InsertRequest, sink: ::grpcio::UnarySink<super::kvserver::InsertResponse>);
    fn update(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::UpdateRequest, sink: ::grpcio::UnarySink<super::kvserver::UpdateResponse>);
    fn delete(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::DeleteRequest, sink: ::grpcio::UnarySink<super::kvserver::DeleteResponse>);
    fn scan(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::ScanRequest, sink: ::grpcio::UnarySink<super::kvserver::ScanResponse>);
    fn scan_all(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::ScanAllRequest, sink: ::grpcio::UnarySink<super::kvserver::ScanAllResponse>);
    fn clear(&mut self, ctx: ::grpcio::RpcContext, req: super::kvserver::ClearRequest, sink: ::grpcio::UnarySink<super::kvserver::ClearResponse>);
}

pub fn create_kvdb<S: Kvdb + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_GET, move |ctx, req, resp| {
        instance.get(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_SET, move |ctx, req, resp| {
        instance.set(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_INSERT, move |ctx, req, resp| {
        instance.insert(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_UPDATE, move |ctx, req, resp| {
        instance.update(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_DELETE, move |ctx, req, resp| {
        instance.delete(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_SCAN, move |ctx, req, resp| {
        instance.scan(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KVDB_SCAN_ALL, move |ctx, req, resp| {
        instance.scan_all(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_KVDB_CLEAR, move |ctx, req, resp| {
        instance.clear(ctx, req, resp)
    });
    builder.build()
}
