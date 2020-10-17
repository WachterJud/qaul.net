//! libqaul RPC compatibility adapter
//!
//! By default `libqaul` is only meant to be used by local Rust
//! clients.  To allow third-party clients to also interact with a
//! running stack, you should use the qrpc bus.  This module exposes
//! some utilities to bind libqaul functions to an rpc server.
//!
//! To write a service to use libqaul, include the client-lib
//! (libqaul-rpc) for type and API configuration.

use crate::QaulRef;
use qrpc_sdk::{default_socket_path, error::RpcResult, RpcSocket, Service};
use async_std::sync::Arc;

/// A pluggable RPC server that wraps around libqaul
///
/// Initialise this server with a fully initialised [`Qaul`] instance.
/// You will lose access to this type once you start the RPC server.
/// Currently there is no self-management interface available via
/// qrpc.
///
pub struct RpcServer {
    inner: QaulRef,
    socket: Arc<RpcSocket>,
}

impl RpcServer {
    /// Wrapper around `new` with `default_socket_path()`
    pub async fn start_default(inner: QaulRef) -> RpcResult<Self> {
        let (addr, port) = default_socket_path();
        Self::new(inner, addr, port).await
    }

    pub async fn new(inner: QaulRef, addr: &str, port: u16) -> RpcResult<Self> {
        let socket = RpcSocket::connect(addr, port).await?;
        let _self = Self { inner, socket };
        Ok(_self)
    }
}
