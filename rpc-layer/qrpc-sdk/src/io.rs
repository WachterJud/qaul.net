//! I/O utility module

use crate::{builders::_internal, error::RpcResult, types::rpc_message};
use async_std::{net::TcpStream, prelude::*};
use byteorder::{BigEndian, ByteOrder};
use identity::Identity;

/// A message buffer to send or receive
pub struct Message {
    pub id: Identity,
    pub addr: String,
    pub data: Vec<u8>,
}

/// Read a framed message from a socket
pub async fn recv(s: &mut TcpStream) -> RpcResult<Message> {
    let mut len_buf = vec![0; 8];
    s.read_exact(&mut len_buf).await?;
    let len = BigEndian::read_u64(&len_buf);

    let mut data = vec![0; len as usize];
    s.read_exact(&mut data).await?;

    // Parse the carrier message type
    let (id, addr, data) = _internal::from(data)?;
    Ok(Message { id, addr, data })
}

/// Send a message with frame
pub async fn send(s: &mut TcpStream, msg: Message) -> RpcResult<()> {
    // Serialise into carrier message type
    let Message { id, addr, data } = msg;
    let mut msg_buf = _internal::to(id, addr, data);

    let mut buffer = vec![0; 8];
    BigEndian::write_u64(&mut buffer, msg_buf.len() as u64);
    buffer.append(&mut msg_buf);

    Ok(s.write_all(&buffer).await?)
}
