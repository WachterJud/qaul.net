//! Wrapper module to read and write to and from an RPC socket
//!

use socket2::{Domain, SockAddr, Socket, Type};
use std::{io::Result, path::Path};

pub(crate) struct RpcSocket {
    inner: Socket,
}

impl RpcSocket {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut inner = Socket::new(
            Domain::unix(),
            Type::seqpacket(), // this _may_ not be supported on MacOS
            None,
        )?;
        inner.connect(&SockAddr::unix(path)?)?;

        Ok(Self { inner })
    }
}
