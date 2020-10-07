//! An extensible rpc message broker for the libqaul ecosystem.

use qrpc_sdk::{default_socket_path, RpcSocket};
use std::{path::PathBuf, sync::Arc};

/// Hold the main broker state
pub struct Broker {
    sock: Arc<RpcSocket>,
}

impl Broker {
    pub fn new<P: Into<PathBuf>>(path: Option<P>) -> Self {
        let path = path.map(|p| p.into()).unwrap_or(default_socket_path());
        let sock = RpcSocket::new(path).unwrap();
        Self { sock }
    }
}

// #[test]
// fn make_it_just_work_please() {
//     use capnp::{message::Builder, serialize_packed};
//     use qrpc_sdk::types::rpc_broker::service;

//     let mut msg = Builder::new_default();
//     let mut service = msg.init_root::<service::Builder>();

//     let d = "This is a test service to see how the RPC layer works";

//     service.set_name("net.qaul.test-service");
//     service.set_version(1);
//     service.set_description(d.clone());

//     let mut buffer = vec![];
//     serialize_packed::write_message(&mut buffer, &msg).unwrap();

//     //// Now test our de-serialisation logic
//     let reader = UtilReader::new(buffer).unwrap();
//     let parsed: service::Reader = reader.get_root().unwrap();

//     assert_eq!(parsed.get_name().unwrap(), "net.qaul.test-service");
//     assert_eq!(parsed.get_description().unwrap(), d);
//     assert_eq!(parsed.get_version(), 1);
// }

/*
Stuff I need

service -> service
service -> libqaul
libqaul -> service (reply, push/ subscription)


Each service has two parts: service core, and service client lib

service client lib:

- no logic
- defines the API and types with capn proto

service core:

- all the logic
- no types
- connects to the broker to advertise it's capabilities

Service advertisement

- name
- hash id
- capabilities

*/
