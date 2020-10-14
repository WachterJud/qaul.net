//! Read the basic protocol
//!
//! A message can either be addressed to the message broker itself, or
//! to another component on the bus.  Messages to the broker have the
//! address `net.qaul._broker`, while all other addresses need to be
//! looked up in the component table.

use crate::Broker;
use async_std::task;
use identity::Identity;
use qrpc_sdk::{
    builders,
    io::MsgReader,
    rpc::{
        capabilities::{self, Which},
        register, unregister,
    },
    types::{rpc_message, service},
    PosixAddr, PosixSocket, RpcSocket,
};
use std::sync::Arc;
use tracing::{debug, error, info};

type CapReader = MsgReader<'static, capabilities::Reader<'static>>;
type RpcReader = MsgReader<'static, rpc_message::Reader<'static>>;

pub(crate) enum Message {
    Command(Vec<u8>),
    Relay { addr: String, data: Vec<u8> },
}

/// Parse a carrier message into a typed command for the broker
///
/// Either it has been addressed to the broker, meaning that it needs
/// to handle the incoming message, or it will simply relay it to a
/// known service
pub(crate) fn parse_carrier(buf: Vec<u8>) -> Option<Message> {
    let reader: RpcReader = MsgReader::new(buf).ok()?;
    let cardr = reader.get_root().ok()?;

    let addr = cardr.get_addr().ok()?.to_string();
    let data = cardr.get_data().ok()?.to_vec();

    Some(match addr.as_str() {
        "net.qaul._broker" => Message::Command(data),
        _ => Message::Relay { addr, data },
    })
}

/// Get new service name from a registry message
#[inline]
fn parse_register(r: register::Reader) -> Option<String> {
    let sr: service::Reader = r.get_service().ok()?;
    sr.get_name().map(|s| s.to_string()).ok()
}

/// Get hash_id from an unregistry message
#[inline]
fn parse_unregister(u: unregister::Reader) -> Option<Identity> {
    u.get_hash_id()
        .map(|id| Identity::from_string(&id.to_string()))
        .ok()
}

/// Handle a commande meant for the message broker
#[inline]
pub(crate) fn handle_broker_cmd(
    broker: Arc<Broker>,
    rpc: Arc<RpcSocket>,
    src_addr: Arc<PosixAddr>,
    sock: &PosixSocket,
    buf: Vec<u8>,
) -> Option<()> {
    let capr: CapReader = MsgReader::new(buf).ok()?;

    match capr.get_root().ok()?.which() {
        Ok(Which::Register(Ok(reg))) => {
            let name = parse_register(reg)?;
            info!("Registering new service: `{}`", name);

            task::block_on(async {
                if let Some(id) = broker.add_new_service(name, Arc::clone(&src_addr)).await {
                    rpc.send_raw(sock, builders::resp_id(id), Some(src_addr.as_ref()));
                    Some(())
                } else {
                    debug!("Error: failed to register service!");
                    None
                }
            })
        }
        Ok(Which::Unregister(Ok(unreg))) => {
            let id = parse_unregister(unreg)?;
            task::block_on(async { broker.remove_service_by_id(id).await });
            rpc.send_raw(sock, builders::resp_bool(true), Some(src_addr.as_ref()));
            Some(())
        }
        Ok(Which::Upgrade(Ok(_))) => todo!(),
        _ => {
            error!("Invalid capability set; dropping connection");
            None
        }
    }
}
