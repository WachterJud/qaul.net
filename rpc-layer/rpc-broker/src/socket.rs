//! Wrapper module to read and write to and from an RPC socket
//!

use async_std::{sync::Arc, task};
use byteorder::{BigEndian, ByteOrder};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    io::Result,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub(crate) struct RpcSocket {
    inner: Socket,
    run: AtomicBool,
}

impl RpcSocket {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        let run = AtomicBool::from(true);
        let mut inner = Socket::new(
            Domain::unix(),
            Type::seqpacket(), // this _may_ not be supported on MacOS
            None,
        )?;
        inner.connect(&SockAddr::unix(path)?)?;
        inner.listen(128)?;

        Ok(Arc::new(Self { inner, run }).spawn())
    }

    /// Spawn a worker to read messages from the socket
    fn spawn(self: Arc<Self>) -> Arc<Self> {
        let s = Arc::clone(&self);
        task::spawn(async move {
            while s.run.load(Ordering::Relaxed) {
                while let Ok((sock, addr)) = s.inner.accept() {
                    let arc = Arc::clone(&s);
                    task::spawn(arc.handle_socket(sock, addr));
                }
            }
        });
        self
    }

    /// Handle one socket connection
    async fn handle_socket(self: Arc<Self>, sock: Socket, addr: SockAddr) {
        while self.run.load(Ordering::Relaxed) {
            let len = match read_length(&self.inner) {
                Some(len) => len,
                None => continue,
            };

            let buf = vec![0, len];
            if self.inner.recv(&mut buf).is_err() && continue {}

            
        }
    }
}

/// Read 8 bytes to get the message length
fn read_length(sock: &Socket) -> Option<usize> {
    let mut len_buf = [0; 8];
    sock.recv(&mut len_buf).ok()?;
    BigEndian::read_u64(&len_buf)
}
