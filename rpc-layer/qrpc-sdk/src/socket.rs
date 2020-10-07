//! An internal abstraction over the RPC socket

use crate::{
    builders,
    errors::{RpcError, RpcResult},
};
use async_std::{future, sync::Arc, task};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    future::Future,
    io::Result,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Get the location the qrpc socket _should_ be by default
///
/// This default can be overridden, though!  It's safer to make this
/// option configurable for the user, instead of only relying on the
/// default.
pub fn default_socket_path() -> PathBuf {
    PathBuf::from("/run/user/1000/qrpc.socket")
}

/// A qrpc connection wrapper
///
/// This type wraps a UNIX socket connection to a remote client.  By
/// default it is configured in client-only mode, meaning that the
/// only time it listens for incoming messages is when waiting for a
/// reply from the rpc broker, libqaul, or another service.  To
/// pro-actively reply to incoming requests (for example, if you want
/// that your service can be used by other services)
pub struct RpcSocket {
    inner: Socket,
    addr: SockAddr,
    run: AtomicBool,
    listening: AtomicBool,
    timeout: Duration,
}

impl RpcSocket {
    /// Create a new socket to the RPC system
    ///
    /// To listen for new connections you need to explicitly call
    /// `listen(...)`, otherwise it will only act as a sending socket,
    /// where each reply is meant for one request.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        Self::with_duration(path, Duration::from_secs(5))
    }

    /// Create a new socket with an explicit timeout duration
    ///
    /// Setup is the same as when calling `new`, except that you can
    /// choose an explicit timeout, instead of the default.
    pub fn with_duration<P: AsRef<Path>>(path: P, timeout: Duration) -> Result<Arc<Self>> {
        let addr = SockAddr::unix(path)?;
        let mut inner = Socket::new(
            Domain::unix(),
            Type::seqpacket(), // this _may_ not be supported on MacOS
            None,
        )?;
        inner.connect(&addr)?;

        Ok(Arc::new(Self {
            inner,
            addr,
            timeout,
            run: AtomicBool::from(true),
            listening: AtomicBool::from(false),
        }))
    }

    /// Start listening for connections with a future
    ///
    /// Incoming messages need to be parsed by your service, and then
    /// replied to.
    pub fn listen<F>(self: &Arc<Self>, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.listening.fetch_or(true, Ordering::Relaxed);
        task::spawn(fut);
    }

    /// Send a binary payload message to a specific service.
    ///
    /// This function needs to be called by your service when mapping
    /// your public API to the RPC layer.  Internally all requests
    /// will be proxied, and parsed by your service backend.
    ///
    /// Use the message builder functions available in [`io`] to
    /// construct a correctly packed and compressed message.
    ///
    /// In order to react to the response sent by the other side, you
    /// need to provide a future to be run.
    ///
    /// [`io`]: ./io/index.html
    pub async fn send_msg<F, T, S>(
        self: &Arc<Self>,
        target: S,
        msg: Vec<u8>,
        handle: F,
    ) -> RpcResult<T>
    where
        F: Future<Output = RpcResult<T>> + Send + 'static,
        S: Into<String>,
    {
        let msg = builders::_internal_to(target.into(), msg);
        let _self = Arc::clone(self);
        self.with_timeout(async move {
            // let len = match _self.inner.send_to(&msg, &_self.addr) {
            //     Ok(l) => l,
            //     Err(e) => return Err(RpcError::Other(e.to_string())),
            // };

            todo!()
        })
        .await;

        todo!()
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

    /// Drive a future to completion with a timeout
    async fn with_timeout<T, F>(&self, fut: F) -> RpcResult<T>
    where
        F: Future<Output = T> + Send + 'static,
    {
        future::timeout(self.timeout.clone(), fut)
            .await
            .map_err(|_| RpcError::Timeout)
    }
}
