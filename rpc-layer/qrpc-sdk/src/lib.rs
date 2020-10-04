//! # qaul rpc sdk
//!
//! This library provides the basic capabilities of interacting with a
//! qrpc-broker, and other qaul services.  These docs outline API
//! usage and concrete types.  For an overview of concepts, consult
//! the [contributors manual][manual]
//!
//! [manual]: https://docs.qaul.net/contributors/technical/rpc-layer
//!
//! ## Using this sdk
//!
//! In order to interact with a running qrpc-broker instance your
//! service needs to register itself and it's capabilities.  This
//! mechanism is handled by this sdk.
//!
//! If you want other third-party services to be able to depend on
//! your service you may want to split it into two parts: one part
//! contains the service logic, the other type and RPC interface
//! information.  This way a third-party service can include your
//! services' type library (usually called `<service name>-rpc`), and
//! gain access to all public RPC functions.
//!
//! If this is not a requirement for you, don't worry about it.

use identity::Identity;

// FIXME: currently the protocols have to be in the root of the crate
// because of [this issue][i] in the capnproto codegen units:
// [i]: https://github.com/capnproto/capnproto-rust/issues/194
pub(crate) mod carrier_capnp {
    #![allow(unused)] // don't bother me pls
    include!(concat!(env!("OUT_DIR"), "/schema/carrier_capnp.rs"));
}

/// A service representation on the qrpc system
pub struct Service {
    name: String,
    version: u16,
    description: String,
    hash_id: Option<Identity>,
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
        }
    }

    /// Register this service with the RPC broker/ libqaul
    pub async fn register(&mut self) -> Option<()> {
        None
    }
}
