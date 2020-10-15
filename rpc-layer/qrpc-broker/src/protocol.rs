//! Read the basic protocol
//!
//! A message can either be addressed to the message broker itself, or
//! to another component on the bus.  Messages to the broker have the
//! address `net.qaul._broker`, while all other addresses need to be
//! looked up in the component table.

use crate::{ConnMap, ServiceEntry};
use async_std::net::TcpStream;
use identity::Identity;

use qrpc_sdk::{
    builders,
    error::{RpcError, RpcResult},
    io::Message,
    parser::MsgReader,
    rpc::{
        capabilities::{self, Which},
        register,
    },
    types::service,
};

type CapReader = MsgReader<'static, capabilities::Reader<'static>>;

/// Get new service name from a registry message
#[inline]
fn parse_register(r: register::Reader) -> RpcResult<String> {
    let sr: service::Reader = r.get_service()?;
    Ok(sr.get_name().map(|s| s.to_string())?)
}

/// Handle a commande meant for the message broker
#[inline]
pub(crate) async fn broker_command(
    req_id: Identity,
    stream: &TcpStream,
    buf: Vec<u8>,
    conns: &ConnMap,
) -> RpcResult<Message> {
    let capr: CapReader = MsgReader::new(buf)?;

    let mut conns = conns.write().await;
    match capr.get_root()?.which() {
        Ok(Which::Register(Ok(reg))) => {
            let name = parse_register(reg)?;
            let id = Identity::random();

            let entry = ServiceEntry {
                addr: name.clone(),
                io: stream.clone(),
                id,
            };

            conns.insert(name.clone(), entry);
            Ok(Message {
                id: req_id,
                addr: name,
                data: builders::resp_id(id),
            })
        }
        Ok(_) => todo!(),
        _ => Err(RpcError::EncoderFault(
            "failed to parse capability message!".into(),
        )),
    }
}
