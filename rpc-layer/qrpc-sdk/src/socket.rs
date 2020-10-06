//! An internal abstraction over the RPC socket

use async_std::{sync::Arc, task};
use byteorder::{BigEndian, ByteOrder};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    future::Future,
    io::Result,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct RpcSocket {
    inner: Socket,
    run: AtomicBool,
    listening: AtomicBool,
}

impl RpcSocket {
    /// Create a new socket to the RPC system
    ///
    /// To listen for new connections you need to explicitly call
    /// `listen(...)`, otherwise it will only act as a sending socket,
    /// where each reply is meant for one request.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        let run = AtomicBool::from(true);
        let listening = AtomicBool::from(false);
        let mut inner = Socket::new(
            Domain::unix(),
            Type::seqpacket(), // this _may_ not be supported on MacOS
            None,
        )?;
        inner.connect(&SockAddr::unix(path)?)?;

        Ok(Arc::new(Self {
            inner,
            run,
            listening,
        }))
    }

    /// Start listening for connections with a future
    pub fn listen<F>(self: &Arc<Self>, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.listening.fetch_or(true, Ordering::Relaxed);
        task::spawn(fut);
    }

    /// Check if the socket is still running
    ///
    /// Use this function in your service's listening code to
    /// determine whether the connection should be shut-down
    pub fn running(&self) -> bool {
        self.run.load(Ordering::Relaxed)
    }

    /// Query whether this socket is listening for connections
    pub fn listening(&self) -> bool {
        self.listening.load(Ordering::Relaxed)
    }
}
