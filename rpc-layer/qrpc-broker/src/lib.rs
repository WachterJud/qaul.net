//! An extensible rpc message broker for the libqaul ecosystem.

use async_std::{sync::RwLock, task};
use identity::Identity;
use qrpc_sdk::{
    builders, default_socket_path,
    error::RpcError,
    io::MsgReader,
    rpc::capabilities::{self, Which},
    types::service,
    PosixAddr, PosixSocket, RpcSocket,
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, error, info};

type CapReader = MsgReader<'static, capabilities::Reader<'static>>;

pub struct ServiceEntry {
    addr: Arc<PosixAddr>,
    id: Identity,
}

/// Hold the main broker state
pub struct Broker {
    sock: Arc<RpcSocket>,
    connections: Arc<RwLock<BTreeMap<String, ServiceEntry>>>,
}

impl Broker {
    pub fn new() -> Arc<Self> {
        let sock = RpcSocket::create(default_socket_path()).unwrap();
        let connections = Default::default();

        let this = Arc::new(Self { sock, connections });

        let _this = Arc::clone(&this);
        this.sock.start_server(move |socket, addr| {
            task::block_on(async { _this.handle_connection(socket, addr).await });
        });

        this
    }

    /// Handle connections from a single incoming socket
    async fn handle_connection(self: &Arc<Self>, sock: Arc<RpcSocket>, src_addr: PosixAddr) {
        info!("Receiving connection to: {:?}", src_addr);
        let src_addr = Arc::new(src_addr);

        loop {
            let (_dst_addr, buffer) = match sock.recv() {
                Ok(a) => {
                    debug!("Receiving incoming message...");
                    a
                }
                Err(RpcError::ConnectionFault) => {
                    debug!("Error: Connection dropped");
                    break;
                }
                Err(_) => {
                    debug!("Error: Invalid payload");
                    continue;
                }
            };

            let capr: CapReader = match MsgReader::new(buffer) {
                Ok(r) => r,
                Err(_) => {
                    sock.send_raw(builders::resp_bool(false), None);
                    continue;
                }
            };

            debug!("Parsing message");

            match capr.get_root().unwrap().which() {
                // Registering means needing to insert the service and report either ok or not
                Ok(Which::Register(Ok(reg))) => {
                    let sr: service::Reader = match reg.get_service() {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                    let name = match sr.get_name() {
                        Ok(n) => n,
                        Err(_) => {
                            sock.send_raw(builders::resp_bool(false), Some(src_addr.as_ref()));
                            continue;
                        }
                    };
                    info!("Registering new service: {}", name);

                    if let Some(id) = self
                        .add_new_service(name.to_string(), Arc::clone(&src_addr))
                        .await
                    {
                        sock.send_raw(builders::resp_id(id), Some(src_addr.as_ref()));
                    } else {
                        sock.send_raw(builders::resp_bool(false), Some(src_addr.as_ref()));
                        continue;
                    }
                }
                Ok(Which::Unregister(Ok(unreg))) => {
                    let id = match unreg.get_hash_id() {
                        Ok(n) => n,
                        Err(_) => {
                            sock.send_raw(builders::resp_bool(false), Some(src_addr.as_ref()));
                            continue;
                        }
                    };

                    let id = Identity::from_string(&id.to_string());
                    self.remove_service_by_id(id).await;
                    sock.send_raw(builders::resp_bool(true), Some(src_addr.as_ref()));
                }
                Ok(Which::Upgrade(Ok(_))) => todo!(),
                _ => {
                    error!("Invalid capability set; dropping connection");
                    continue;
                }
            }
        }
    }

    /// Insert a new service to the map, return None if it already exists
    async fn add_new_service(
        self: &Arc<Self>,
        name: String,
        addr: Arc<PosixAddr>,
    ) -> Option<Identity> {
        let mut conn = self.connections.write().await;
        if conn.contains_key(&name) {
            return None;
        }

        let id = Identity::random();
        conn.insert(name, ServiceEntry { addr, id });
        Some(id)
    }

    async fn get_service(self: &Arc<Self>, name: &String) -> Option<Arc<PosixAddr>> {
        self.connections
            .read()
            .await
            .get(name)
            .map(|entry| Arc::clone(&entry.addr))
    }

    async fn remove_service_by_id(self: &Arc<Self>, id: Identity) -> Option<()> {
        let mut conn = self.connections.write().await;
        if let Some(ref name) = conn
            .iter()
            .find(|(_, entry)| entry.id == id)
            .map(|(name, _)| name.clone())
        {
            conn.remove(name);
            Some(())
        } else {
            None
        }
    }
}
