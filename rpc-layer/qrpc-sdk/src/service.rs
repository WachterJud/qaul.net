use crate::errors::RpcResult;
use crate::{builders, RpcSocket};
use identity::Identity;
use std::sync::Arc;

/// Access the socket stored in a service
fn _socket(s: &Service) -> &Arc<RpcSocket> {
    s.socket.as_ref().unwrap()
}

/// A service representation on the qrpc system
///
/// Use this struct to handle RPC connections to the network, and to
/// update any data you want your service to broadcast to other
/// participants on the QRPC system.
pub struct Service {
    name: String,
    version: u16,
    description: String,
    hash_id: Option<Identity>,
    socket: Option<Arc<RpcSocket>>,
}

impl Service {
    /// Create a new service without hash_id
    ///
    /// The `hash_id` field will be filled in by the remote RPC server
    /// after calling `register()`.
    pub fn new<S: Into<String>>(name: S, version: u16, description: S) -> Self {
        Self {
            name: name.into(),
            version,
            description: description.into(),
            hash_id: None,
            socket: None,
        }
    }

    /// Register this service with the RPC broker/ libqaul
    pub async fn register(&mut self, socket: Arc<RpcSocket>) -> RpcResult<Identity> {
        self.socket = Some(socket);
        let (target, reg_msg) = builders::register(&self);
        use crate::rpc::register;
        _socket(self)
            .send_msg(target, reg_msg, |reader| {
                let r: register::Reader = reader.get_root().unwrap();
                todo!()
            })
            .await
    }

    /// Get the `hash_id` field of this service, if it's set
    pub fn hash_id(&self) -> Option<Identity> {
        self.hash_id
    }
}

/// An external service that can be connected to
///
/// In order to use the function-set from an external service, your
/// application needs to include it's client-lib (usually named
/// `<service>-rpc`) which provides a strongly typed API that
/// abstracts away the RPC protocol logic.
///
/// Any service that should be connectable needs to implement this
/// trait.  Any service API object also needs to implement the
/// `Default` trait so that the sdk internally can create a default of
/// it, then call `establish_connection()` to fully initialise it.
#[async_trait::async_trait]
pub trait ServiceConnector: Default {
    /// Start a connection to the service backend
    async fn establish_connection(self: Arc<Self>) -> RpcResult<()>;
    /// Terminate the connection to the service backend
    async fn terminate_connection(self: Arc<Self>) -> RpcResult<()>;
}
