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


pub mod io;

use identity::Identity;

// FIXME: currently the protocols have to be in the root of the crate
// because of [this issue][i] in the capnproto codegen units:
// [i]: https://github.com/capnproto/capnproto-rust/issues/194
pub(crate) mod carrier_capnp {
    #![allow(unused)] // don't bother me pls
    include!(concat!(env!("OUT_DIR"), "/schema/carrier_capnp.rs"));
}

/// Basic qrpc trasmission types
///
/// This interface is exposed to let other parts of the qrpc ecosystem
/// parse and generate these types.  When using this library directly,
/// try to avoid using them.  Use the main type interface documented
/// in the root of the crate instead.
pub mod types {
    pub use crate::carrier_capnp::service;
}

/// Unterlying RPC message types
///
/// As with the data types used by this crate, try to avoid using them
/// directly.  Instead use the main API of the crate which invoces
/// these types internally
pub mod rpc {
    pub use crate::carrier_capnp::{register, unregister, upgrade};
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
