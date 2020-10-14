//! An extensible rpc message broker for the libqaul ecosystem.

mod exit;
mod protocol;

use crate::protocol::Message;
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
use tracing::{debug, error, info, trace};

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
        this.sock.start_server(move |rpc, sock, addr| {
            let rpc = Arc::clone(&rpc);
            let t = Arc::clone(&_this);
            task::spawn(async move { t.handle_connection(rpc, sock, addr).await });
        });

        // Make sure we clean up the socket when we exti
        exit::add_shutdown_hooks(Arc::clone(&this));

        this
    }

    /// Handle connections from a single incoming socket
    async fn handle_connection(
        self: Arc<Self>,
        rpc: Arc<RpcSocket>,
        sock: PosixSocket,
        src_addr: PosixAddr,
    ) {
        info!("Receiving connection from: {:?}", src_addr);
        let src_addr = Arc::new(src_addr);

        loop {
            let (_dst_addr, buffer) = match rpc.recv(&sock) {
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

            trace!("Parsing carrier message...");
            match protocol::parse_carrier(buffer) {
                Some(Message::Command(buf)) => {
                    if protocol::handle_broker_cmd(
                        Arc::clone(&self),
                        Arc::clone(&rpc),
                        Arc::clone(&src_addr),
                        &sock,
                        buf,
                    )
                    .is_none()
                    {
                        error!("Failed to execute broker command: skipping!");
                        continue;
                    }
                }
                Some(Message::Relay { addr, data }) => match self.get_service(&addr).await {
                    Some(addr) => rpc.send_raw(&sock, data, Some(addr.as_ref())),
                    None => {
                        rpc.send_raw(&sock, builders::resp_bool(false), Some(src_addr.as_ref()))
                    }
                },
                None => {
                    error!("Failed parsing carrier frame: skipping!");
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
