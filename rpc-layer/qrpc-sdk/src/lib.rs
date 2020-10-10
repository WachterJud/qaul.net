//! A toolkit for writing clients on the qrpc message bus.  This bus
//! is the backbone of the [qaul.net](https://qaul.net) service
//! ecosystem.  With it you can create applications (services) that
//! interact with `libqaul`, and other services on the same message
//! broker.
//!
//! These crate docs describe the API and basic usage.  For an
//! overview of the core concepts of this ecosystem, consult the
//! [contributors manual][manual].
//!
//! Additionally, you can access documentation of the internal
//! utilities by passing `--features internal` to your cargo
//! invocation.  These components are exposed via the API either way,
//! but only documented on demand to not clutter the main
//! documentation.
//! 
//!
//! [manual]: https://docs.qaul.net/contributors/technical/rpc-layer
//!
//! ## Using this sdk
//!
//! In order to interact with a running [`qrpc-broker`] instance your
//! service needs to register itself and it's capabilities.
//!
//! First your service needs a place to save some state, composing
//! different parts of this sdk together to create an app.  You create
//! a [`Service`] and [`RpcSocket`] and connect to the rpc-broker
//! socket.  First you will have to call `register(...)` on the
//! `Service`, before any messages can be relayed to you.
//!
//! [`qrpc-broker`]: ../qrpc_broker/index.html
//! [`Service`]: ./struct.Service.html
//! [`RpcSocket`]: ./struct.RpcSocket.html
//!
//! ```
//! # fn foo() -> Result<(), Box<std::error::Error>> {
//! use qrpc_sdk::{Service, RpcSocket, default_socket_path};
//!
//! let serv = Service::new("com.example.myapp", 1, "A simple app");
//! let sockt = RpcSocket::new(default_socket_path())?;
//! serv.register(sock)?;
//! println!("Service registered! Hash ID: {}", serv.hash_id().unwrap());
//! # }
//! ```
//!
//! Include the client-lib of the component you want to connect to,
//! and call `establish_connection()`, privded by
//! [`ServiceConnector`].  This will establish a connection with the
//! service to verify it's capability set.  Your service will also
//! have to implement this mechanism to be usable by other services on
//! the RPC bus.
//!
//! [`ServiceConnector`]: ./trait.ServiceConnector.html
//!
//! After that you can call functions on the public API type of the
//! component.  You can get a copy of it via your service handle:
//! `service.component("net.qaul.libqaul")`.
//!
//! If you want to see a minimal example of the smallest functional
//! service, see the [`ping`] crate.
//!
//! [`ping`]: https://git.open-communication.net/qaul/qaul.net/-/tree/develop/services%2Fping/

pub mod io;

// FIXME: currently the protocols have to be in the root of the crate
// because of [this issue][i] in the capnproto codegen units:
// [i]: https://github.com/capnproto/capnproto-rust/issues/194
pub(crate) mod base_capnp {
    #![allow(unused)] // don't bother me pls
    include!(concat!(env!("OUT_DIR"), "/schema/base_capnp.rs"));
}
pub(crate) mod types_capnp {
    #![allow(unused)] // don't bother me pls
    include!(concat!(env!("OUT_DIR"), "/schema/types_capnp.rs"));
}
pub(crate) mod cap_capnp {
    #![allow(unused)] // don't bother me pls
    include!(concat!(env!("OUT_DIR"), "/schema/cap_capnp.rs"));
}

/// qrpc message types
///
/// This interface is exposed to let other parts of the qrpc ecosystem
/// parse and generate these types.  When using this library directly,
/// try to avoid using them.  Use the main type interface documented
/// in the root of the crate instead.
#[cfg_attr(not(feature = "internals"), doc(hidden))]
pub mod types {
    pub use crate::base_capnp::rpc_message;
    pub use crate::types_capnp::service;
}

/// RPC message types used by the qrpc-sdk
///
/// As with the data types used by this crate, try to avoid using them
/// directly.  Instead use the main API of the crate which invoces
/// these types internally
#[cfg_attr(not(feature = "internals"), doc(hidden))]
pub mod rpc {
    pub use crate::cap_capnp::{capabilities, register, sdk_reply, unregister, upgrade};
}

mod service;
mod socket;

pub mod builders;
pub mod errors;

pub use service::{Service, ServiceConnector};
pub use socket::{default_socket_path, RpcSocket};

#[cfg_attr(not(feature = "internals"), doc(hidden))]
pub use socket::{SockAddr as PosixAddr, Socket as PosixSocket};
