//! Internal abstraction over the Rpc socket
//!
//! The protocol uses TCP as a transport, meaning that when sending
//! messages, they need to be framed.  The `builder` abstraction takes
//! care of this!  Do not manually frame your messages!

use crate::{
    error::{RpcError, RpcResult},
    io::{self, Message},
};
use async_std::{
    future,
    net::{TcpListener, TcpStream},
    stream::StreamExt,
    sync::{channel, Arc, Mutex, Receiver, Sender},
    task,
};
use identity::Identity;
use std::{
    collections::BTreeMap,
    net::Shutdown,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

type Lock<T> = Arc<Mutex<T>>;

/// Return the default bind location for the qrpc broker socket
pub fn default_socket_path() -> (&'static str, u16) {
    ("localhost", 10222)
}

/// Bi-directional socket connection to a qrpc bus system
///
/// A connection is always between a component on the bus, and the
/// broker.  The broker listens to incoming connections, and relays
/// them.  A component (service, or utility library) can either
/// operate only in sending mode, or listen as well, so that it can be
/// used as a dependency by other services.  The sending socket is
/// used as a listener, meaning that no specific port needs to be
/// bound for a service.
///
/// When using the `server(...)` constructor you bind a port, when
/// attaching a lambda via `listen(...)` you use the established
/// connection.  In your service code there is no reason to ever use
/// `server(...)`!
///
/// When sending a message, the socket will listen for a reply from
/// the broker on the sending stream, to make sure that return data is
/// properly associated.  You can control the timeout via the
/// `connect_timeout` function.
pub struct RpcSocket {
    stream: Option<TcpStream>,
    listen: Option<Arc<TcpListener>>,
    running: AtomicBool,
    listening: AtomicBool,
    wfm: Lock<BTreeMap<Identity, Sender<Message>>>,
    inc_io: (Sender<Message>, Receiver<Message>),
    timeout: Duration,
}

impl RpcSocket {
    /// Create a client socket that connects to a remote broker
    pub async fn connect(addr: &str, port: u16) -> RpcResult<Arc<Self>> {
        Self::connect_timeout(addr, port, Duration::from_secs(5)).await
    }

    /// Create a client socket with an explicit timeout
    pub async fn connect_timeout(addr: &str, port: u16, timeout: Duration) -> RpcResult<Arc<Self>> {
        let stream = TcpStream::connect(&format!("{}:{}", addr, port)).await?;

        let _self = Arc::new(Self {
            stream: Some(stream),
            listen: None,
            running: true.into(),
            listening: false.into(),
            wfm: Default::default(),
            inc_io: channel(4),
            timeout,
        });

        _self.spawn_incoming();
        Ok(_self)
    }

    /// Attach a permanent listener to the sending stream
    pub async fn listen<F: Fn(Message) + Send + 'static>(self: &Arc<Self>, cb: F) {
        let _self = Arc::clone(self);
        _self.listening.swap(true, Ordering::Relaxed);
        task::spawn(async move {
            while let Some(msg) = _self.inc_io.1.recv().await {
                cb(msg);
            }
        });
    }

    /// Bind a socket to listen for connections
    ///
    /// This function is primarily used by the qrpc-broker and should
    /// not be used in your service code.  To listen for incoming
    /// connections on the outgoing stream (meaning client side), use
    /// `listen(...)`
    pub async fn server<F, D>(addr: &str, port: u16, cb: F, data: D) -> RpcResult<Arc<Self>>
    where
        F: Fn(TcpStream, D) + Send + Copy + 'static,
        D: Send + Sync + Clone + 'static,
    {
        let listen = Arc::new(TcpListener::bind(format!("{}:{}", addr, port)).await?);
        let _self = Arc::new(Self {
            stream: None,
            listen: Some(listen),
            running: true.into(),
            listening: true.into(),
            wfm: Default::default(),
            inc_io: channel(4),
            timeout: Duration::from_secs(5),
        });

        let s = Arc::clone(&_self);
        task::spawn(async move {
            let mut inc = s.listen.as_ref().unwrap().incoming();
            while let Some(Ok(stream)) = inc.next().await {
                if !s.running() {
                    break;
                }

                let d = data.clone();
                task::spawn(async move { cb(stream, d) });
            }

            info!("Terminating rpc accept loop...");
        });

        Ok(_self)
    }

    /// Handle the incoming side of the stream connection
    ///
    /// When acting as a server this is simple: all messages can be
    /// received at the same point, spawning tasks for each connection
    /// to not mix things up.  On the client side this is harder.  We
    /// need to listen for incoming messages after sending one, so
    /// that we can handle the return data.  But we also need to
    /// generally handle incoming messages.  To avoid having to peek
    /// into the socket periodically to check if a message has
    /// arrived, this mechanism uses channels, and an enum type to
    /// associate message IDs.
    fn spawn_incoming(self: &Arc<Self>) {
        let _self = Arc::clone(self);
        task::spawn(async move {
            let mut sock = _self.stream.clone().unwrap();
            while _self.running.load(Ordering::Relaxed) {
                let msg = match io::recv(&mut sock).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed reading message: {}", e.to_string());
                        continue;
                    }
                };

                let id = msg.id;
                let mut wfm = _self.wfm.lock().await;
                match wfm.get(&id) {
                    Some(sender) => sender.send(msg).await,
                    None => _self.inc_io.0.send(msg).await,
                }

                wfm.remove(&id);
            }
        });
    }

    /// Send a message to the other side of this stream
    ///
    /// This function is meant to be used by qrpc clients that only
    /// have a single connection stream to the broker.  If you wanted
    /// to write an alternative message broker, you have to use the
    /// [`io`] utilities directly (as the `qrpc-broker` crate does)!
    ///
    /// After sending a message this function will wait for a reply
    /// and parse the message for you.  You must provide a conversion
    /// lambda so that the types can be extracted from the message
    /// type that the SDK receives.
    ///
    /// [`io`]: ./io/index.html
    pub async fn send<T, F>(self: &Arc<Self>, msg: Message, convert: F) -> RpcResult<T>
    where
        F: Fn(Message) -> RpcResult<T>,
    {
        // Insert a receive hook for the message we are about to send
        let id = msg.id;
        let (tx, rx) = channel(1);
        self.wfm.lock().await.insert(id, tx);

        // Send off the message...
        let mut s = self.stream.clone().unwrap();
        io::send(&mut s, msg).await?;

        // Wait for a reply
        future::timeout(self.timeout, async move {
            match rx.recv().await {
                Some(msg) => convert(msg),
                None => Err(RpcError::ConnectionFault(
                    "No message with matching ID received!".into(),
                )),
            }
        })
        .await?
    }

    /// Terminate all workers associated with this socket
    pub fn shutdown(self: &Arc<Self>) {
        self.running.swap(false, Ordering::Relaxed);
        if let Some(ref s) = self.stream {
            s.shutdown(Shutdown::Both).unwrap();
        }
    }

    /// Get the current running state
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the current listening state
    pub fn listening(&self) -> bool {
        self.listening.load(Ordering::Relaxed)
    }
}
