//! A set of message builder utilities
//!
//! It's recommended to write similar abstraction layers in your own
//! crate, so to make it easy for other third-party developers to use
//! your service's APIs and types as easily as possible.
//!
//! In your service you will likely not need to consume this API.  It
//! is included for debugging purposes.

use crate::{
    io::Message,
    parser::{MsgReader, Result},
    rpc::{capabilities as cap, sdk_reply as repl},
    Service,
};
use capnp::{message::Builder as Bld, serialize_packed};
use identity::Identity;

/// Generate an enveloped registry message for this service
pub fn register(_service: &Service) -> Result<Message> {
    let mut msg = Bld::new_default();

    let cap = msg.init_root::<cap::Builder>();
    let reg = cap.init_register();
    let mut serv = reg.init_service();

    serv.set_name(&_service.name);
    serv.set_description(&_service.description);
    serv.set_version(_service.version as i16);

    let mut buffer = vec![];
    serialize_packed::write_message(&mut buffer, &msg)?;

    Ok(_internal::make_carrier("net.qaul._broker".into(), buffer))
}

/// Generate a simple boolean return payload
pub fn resp_bool(b: bool) -> Vec<u8> {
    let mut msg = Bld::new_default();
    let mut reply = msg.init_root::<repl::Builder>();
    reply.set_success(b);

    let mut buffer = vec![];
    serialize_packed::write_message(&mut buffer, &msg).unwrap();
    buffer
}

/// Generate a simple ID return payload
pub fn resp_id(id: Identity) -> Vec<u8> {
    let mut msg = Bld::new_default();
    let mut reply = msg.init_root::<repl::Builder>();
    reply.set_hash_id(&id.to_string());

    let mut buffer = vec![];
    serialize_packed::write_message(&mut buffer, &msg).unwrap();
    buffer
}

/// Take a buffer of data and turn it into a reader for Capability messages
pub fn parse_rpc_msg(buffer: Vec<u8>) -> Result<MsgReader<'static, cap::Reader<'static>>> {
    MsgReader::new(buffer)
}

/// This module is only included for debugging reasons.  There's
/// basically no reason to call this function directly.
#[cfg_attr(not(feature = "internals"), doc(hidden))]
pub mod _internal {
    use crate::{error::RpcResult, io::Message, parser::MsgReader, types::rpc_message, Identity};
    use byteorder::{BigEndian, ByteOrder};
    use capnp::{message::Builder as Bld, serialize_packed};

    /// Create a carrier frame with address and a unique ID
    pub fn make_carrier(addr: String, data: Vec<u8>) -> Message {
        let id = Identity::random();
        Message { id, addr, data }
    }

    /// Take address and data and turn it into a basic rpc message
    pub fn to(cmsg: Message) -> Vec<u8> {
        let Message { id, addr, data } = cmsg;

        let mut msg = Bld::new_default();
        let mut carrier = msg.init_root::<rpc_message::Builder>();
        carrier.set_id(&id.to_string());
        carrier.set_addr(&addr);
        carrier.set_data(&data);

        let mut buffer = vec![];
        serialize_packed::write_message(&mut buffer, &msg).unwrap();
        buffer
    }

    /// Read an rpc message from the socket
    ///
    /// Feel free to use this function in your service code.  The
    /// first field in the tuple is the destination address, the
    /// second is the data payload.
    pub fn from(buffer: Vec<u8>) -> RpcResult<(Identity, String, Vec<u8>)> {
        let msg = MsgReader::new(buffer)?;
        let carrier: rpc_message::Reader = msg.get_root().unwrap();
        let id = carrier.get_id()?;
        let addr = carrier.get_addr()?;
        let data = carrier.get_data()?;

        Ok((
            Identity::from_string(&id.to_string()),
            addr.to_string(),
            data.to_vec(),
        ))
    }
}
