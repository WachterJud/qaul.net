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
            RpcSocket::server(addr, port, |stream, data| reader_loop(stream, data), con).await?
        };

        Ok(Arc::new(Self { _sock, _conns }))
    }
}

fn reader_loop(mut stream: TcpStream, data: ConnMap) {
    task::block_on(async {
        loop {
            if let Err(e) = handle_packet(&mut stream, &data).await {
                warn!(
                    "Error occured while accepting packet: {}; dropping stream",
                    e
                );
                break;
            }
        }
    });
}

async fn handle_packet(s: &mut TcpStream, conns: &ConnMap) -> RpcResult<()> {
    let Message { id, addr, data } = io::recv(s).await?;
    match addr.as_str() {
        "net.qaul._broker" => {
            debug!("Message addressed to broker; handling!");
            let msg = protocol::broker_command(id, &s, data, &conns).await?;
            io::send(s, msg).await?;
            Ok(())
        }
        _ => {
            debug!("Message addressed to bus component; looking up stream!");
            let mut t_stream = match conns.read().await.get(&addr).map(|s| s.io.clone()) {
                Some(s) => s,
                None => {
                    warn!("Requested component does not exist no qrpc bus!");
                    io::send(
                        s,
                        Message {
                            id,
                            addr: "<unknown>".into(),
                            data: builders::resp_bool(false),
                        },
                    )
                    .await?;
                    return Ok(());
                }
            };

            // If we reach this point, we can send
            // the relay message
            io::send(&mut t_stream, Message { id, addr, data }).await?;
            Ok(())
        }
    }
}
