//! An extensible rpc message broker for the libqaul ecosystem.

#[macro_use]
extern crate tracing;

mod protocol;

use async_std::{net::TcpStream, sync::RwLock, task};
use identity::Identity;
use qrpc_sdk::{
    builders, default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket,
};
use std::{collections::BTreeMap, sync::Arc};

#[allow(unused)]
pub(crate) struct ServiceEntry {
    pub(crate) id: Identity,
    pub(crate) addr: String,
    pub(crate) io: TcpStream,
}

type ConnMap = Arc<RwLock<BTreeMap<String, ServiceEntry>>>;

/// Hold the main broker state
pub struct Broker {
    _sock: Arc<RpcSocket>,
    _conns: ConnMap,
}

impl Broker {
    pub async fn new() -> RpcResult<Arc<Self>> {
        let (addr, port) = default_socket_path();
        let _conns = Default::default();

        let _sock = {
            let con = Arc::clone(&_conns);
            RpcSocket::server(
                addr,
                port,
                |stream, data| {
                    async fn handle_connection(mut s: TcpStream, conns: ConnMap) -> RpcResult<()> {
                        let Message { id, addr, data } = io::recv(&mut s).await?;

                        match addr.as_str() {
                            // In case the message was addressed to
                            // the broker, handle the payload, adding
                            // or removing a service from the
                            // connection map as required
                            "net.qaul._broker" => {
                                let msg = protocol::broker_command(id, &s, data, &conns).await?;
                                io::send(&mut s, msg).await?;
                                Ok(())
                            }
                            // If the address is some other component,
                            // look up the sending stream to it and
                            // pass the message along.  Send an error
                            // reply to the sender if this failed.
                            _ => {
                                let mut t_stream =
                                    match conns.read().await.get(&addr).map(|s| s.io.clone()) {
                                        Some(s) => s,
                                        None => {
                                            io::send(
                                                &mut s,
                                                Message {
                                                    id,
                                                    addr: "<unknown>".into(),
                                                    data: builders::resp_bool(false),
                                                },
                                            )
                                            .await?;
                                            return Err(RpcError::NoSuchService(addr.clone()));
                                        }
                                    };

                                // If we reach this point, we can send
                                // the relay message
                                io::send(&mut t_stream, Message { id, addr, data }).await?;
                                Ok(())
                            }
                        }
                    }

                    task::block_on(async move {
                        if let Err(e) = handle_connection(stream, data).await {
                            error!("Error occured while accepting connection: {}", e);
                        }
                    });
                },
                con,
            )
            .await?
        };

        Ok(Arc::new(Self { _sock, _conns }))
    }

    // /// Handle connections from a single incoming socket
    // async fn handle_connection(
    //     self: Arc<Self>,
    //     rpc: Arc<RpcSocket>,
    //     sock: PosixSocket,
    //     src_addr: PosixAddr,
    // ) {
    //     info!("Receiving connection from: {:?}", src_addr);
    //     let src_addr = Arc::new(src_addr);

    //     loop {
    //         let (_dst_addr, buffer) = match rpc.recv(&sock) {
    //             Ok(a) => {
    //                 debug!("Receiving incoming message...");
    //                 a
    //             }
    //             Err(RpcError::ConnectionFault) => {
    //                 debug!("Error: Connection dropped");
    //                 break;
    //             }
    //             Err(_) => {
    //                 debug!("Error: Invalid payload");
    //                 continue;
    //             }
    //         };

    //         trace!("Parsing carrier message...");
    //         match protocol::parse_carrier(buffer) {
    //             Some(Message::Command(buf)) => {
    //                 if protocol::handle_broker_cmd(
    //                     Arc::clone(&self),
    //                     Arc::clone(&rpc),
    //                     Arc::clone(&src_addr),
    //                     &sock,
    //                     buf,
    //                 )
    //                 .is_none()
    //                 {
    //                     error!("Failed to execute broker command: skipping!");
    //                     continue;
    //                 }
    //             }
    //             Some(Message::Relay { addr, data }) => match self.get_service(&addr).await {
    //                 Some(addr) => rpc.send_raw(&sock, data, Some(addr.as_ref())),
    //                 None => {
    //                     rpc.send_raw(&sock, builders::resp_bool(false), Some(src_addr.as_ref()))
    //                 }
    //             },
    //             None => {
    //                 error!("Failed parsing carrier frame: skipping!");
    //                 continue;
    //             }
    //         }
    //     }
    // }
}
