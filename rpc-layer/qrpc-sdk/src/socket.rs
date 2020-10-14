//! An internal abstraction over the RPC socket

use crate::{
    builders,
    error::{RpcError, RpcResult},
    io::MsgReader,
};
use async_std::{future, sync::Arc, task};
use capnp::traits::FromPointerReader;
use socket2::{Domain, SockAddr, Type};
use std::{
    clone::Clone,
    fs,
    future::Future,
    io::Result,
    net::Shutdown,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use tracing::{debug, error, info, warn, trace};

/// A special socket address for a posix socket
///
/// You will usually not have to consume this type directly.  But
/// in case you want to structure your program in a way that you
/// need to know this type, you can access it here, instead of
/// having to also depend on the `socket2` crate.
#[cfg(feature = "internals")]
pub use socket2::SockAddr as PosixAddr;

/// The inner wrapped posix socket
pub use socket2::Socket as PosixSocket;

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
    /// Get access to the raw underlying socket but hide it in the docs
    #[doc(hidden)]
    pub inner: Arc<PosixSocket>,
    _addr: SockAddr,
    run: AtomicBool,
    listening: AtomicBool,
    timeout: Duration,
}

impl RpcSocket {
    fn new_socket<P: AsRef<Path>>(path: P) -> Result<(PosixSocket, SockAddr)> {
        let addr = SockAddr::unix(path)?;
        let socket = PosixSocket::new(
            Domain::unix(),
            Type::seqpacket(), // this _may_ not be supported on MacOS
            None,
        )?;

        Ok((socket, addr))
    }

    /// Connect to an established socket to the RPC system
    ///
    /// To listen for new connections you need to explicitly call
    /// `listen(...)`, otherwise it will only act as a sending socket,
    /// where each reply is meant for one request.
    pub fn connect<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        Self::connect_duration(path, Duration::from_secs(5))
    }

    /// Create a new socket with an explicit timeout duration
    ///
    /// Setup is the same as when calling `new`, except that you can
    /// choose an explicit timeout, instead of the default.
    pub fn connect_duration<P: AsRef<Path>>(path: P, timeout: Duration) -> Result<Arc<Self>> {
        let (inner, _addr) = Self::new_socket(path)?;
        inner.connect(&_addr)?;

        Ok(Arc::new(Self {
            inner: Arc::new(inner),
            _addr,
            timeout,
            run: AtomicBool::from(true),
            listening: AtomicBool::from(false),
        }))
    }

    /// Create a new QRPC socket.  This function is meant for servers
    ///
    /// Because creating the socket is synonymous with listening for
    /// connections on it, this function wraps both `new` (sort of),
    /// and `listen`, meaning that you _must_ provide a closure at
    /// this point.
    pub fn create(path: impl AsRef<Path> + Clone) -> Result<Arc<Self>> {
        let (inner, _addr) = Self::new_socket(path.clone())?;
        if let Err(_) = inner.bind(&_addr) {
            warn!("Existing socket found...killing it first!");
            fs::remove_file(path)?;
            inner.bind(&_addr)?;
        }
        inner.listen(32)?;
        debug!("Setting socket to LISTEN=32");

        let arc = Arc::new(Self {
            inner: Arc::new(inner),
            _addr,
            timeout: Duration::from_secs(5),
            run: AtomicBool::from(true),
            listening: AtomicBool::from(true),
        });

        Ok(arc)
    }

    /// Attach the listening part of the socket with a handle
    pub fn start_server<F>(self: &Arc<Self>, handle: F)
    where
        F: Fn(Arc<Self>, PosixSocket, SockAddr) + Send + 'static,
    {
        info!("Listening for incoming socket connetions...");
        // We spawn a dedicated thread because socket2 is a non-async
        // library and we don't want to accidentally deadlock our
        // whole executor on this code.  Besides, it's kinda the
        // primary hot-path on the qrpc system, so a thread might be
        // warranted.  TODO: look into how async-std can handle this!
        let arc2 = Arc::clone(&self);
        thread::spawn(move || {
            while let Ok((ps, addr)) = arc2.inner.accept() {
                handle(Arc::clone(&arc2), ps, addr);
            }
        });
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
    pub async fn send_with_handle<'s, F: 'static, T, S, M: 's>(
        self: &'s Arc<Self>,
        sock: Arc<PosixSocket>,
        target: S,
        msg: Vec<u8>,
        handle: F,
    ) -> RpcResult<T>
    where
        F: Fn(MsgReader<'s, M>) -> RpcResult<T> + Send,
        S: Into<String>,
        M: FromPointerReader<'s>,
    {
        // First send out the message
        let msg = builders::_internal::to(target.into(), msg);
        trace!("Sending {} bytes", msg.len());
        sock.send(&msg).unwrap();

        // Wait for a reply to handle
        let _self = Arc::clone(self);
        self.with_timeout(async move {
            match builders::_internal::from(sock.as_ref()) {
                // This match is essentially a ? but in a closure
                Ok((_, buf)) => match MsgReader::new(buf).map(|ok| handle(ok)) {
                    Ok(f) => f,
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            }
        })
        .await
        .map_err(|_| RpcError::Other("Serialisation failure!".into()))?
    }

    /// Send a message to an address, without handling responses
    ///
    /// This function internally prepends the length via
    /// `builders::_internal::to()`, so don't do it again in your
    /// code!  Use this function when replying to a service request.
    pub fn send_raw(self: &Arc<Self>, sock: &PosixSocket, msg: Vec<u8>, addr: Option<&SockAddr>) {
        if match addr {
            Some(addr) => sock.send_to(&msg, addr),
            None => sock.send(&msg),
        }
        .unwrap()
            < msg.len()
        {
            error!("Tried sending message, but not all bytes went through!");
        }
    }

    /// Receive a message from this socket
    ///
    /// Returns None if the socket is no longer able to yield messages
    pub fn recv(self: &Arc<Self>, sock: &PosixSocket) -> RpcResult<(String, Vec<u8>)> {
        builders::_internal::from(sock)
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

    pub fn shutdown(&self, sock: &PosixSocket) -> Option<()> {
        sock.shutdown(Shutdown::Both).ok()
    }
}
