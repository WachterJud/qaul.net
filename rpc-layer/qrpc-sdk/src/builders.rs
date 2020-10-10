//! A set of message builder utilities
//!
//! It's recommended to write similar abstraction layers in your own
//! crate, so to make it easy for other third-party developers to use
//! your service's APIs and types as easily as possible.
//!
//! In your service you will likely not need to consume this API.  It
//! is included for debugging purposes.

use crate::Service;
use identity::Identity;

/// Generate an registry message for this service
pub fn register(_service: &Service) -> (String, Vec<u8>) {
    todo!()
}

/// Generate an unregistry message for this service
pub fn unregister(_hash_id: Identity) -> (String, Vec<u8>) {
    todo!()
}

/// Generate an upgrade message for this service
pub fn upgrade(_service: &Service, _hash_id: Identity) -> (String, Vec<u8>) {
    todo!()
}


/// This module is only included for debugging reasons.  There's
/// basically no reason to call this function directly.
#[cfg_attr(not(feature = "internals"), doc(hidden))]
pub mod _internal {
    use crate::{io::MsgReader, types::rpc_message};
    use byteorder::{BigEndian, ByteOrder};
    use capnp::{message::Builder as Bld, serialize_packed};
    use socket2::Socket;

    /// Take address and data and turn it into a basic rpc message
    pub fn to(addr: String, data: Vec<u8>) -> Vec<u8> {
        let mut msg = Bld::new_default();
        let mut carrier = msg.init_root::<rpc_message::Builder>();
        carrier.set_addr(&addr);
        carrier.set_data(&data);

        let mut buffer = vec![];
        serialize_packed::write_message(&mut buffer, &msg).unwrap();

        let len = buffer.len();
        let mut message = vec![8];
        BigEndian::write_u64(&mut message, len as u64);

        message.append(&mut buffer);
        message
    }

    /// Read an rpc message from the socket
    ///
    /// Feel free to use this function in your
    ///
    /// The first field in the tuple is the destination address, the
    /// second is the data payload.
    pub fn from(socket: &Socket) -> (String, Vec<u8>) {
        let mut len = vec![0; 8];
        loop {
            let (l, _) = socket.peek_from(&mut len).unwrap();
            if l == 8 {
                break;
            }
        }

        let (_, _) = socket.recv_from(&mut len).unwrap();
        let len = BigEndian::read_u64(&len);
        let mut buffer = vec![0; len as usize];
        socket.recv_from(&mut buffer).unwrap();

        let msg = MsgReader::new(buffer).unwrap();
        let carrier: rpc_message::Reader = msg.get_root().unwrap();
        let addr = carrier.get_addr().unwrap();
        let data = carrier.get_data().unwrap();

        (addr.to_string(), data.to_vec())
    }
}
