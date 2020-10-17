//! A type fascade for `libqaul`
//!
//! Using this crate directly is usually not neccessary, instead you
//! should use `libqaul-rpc` to pull in the RPC adapter for the API.
//!
//! To learn more about how to write components for the qaul.net rpc
//! system (qrpc), check out the `qrpc-sdk` crate documentation.
//!
//! All types in this crate should be `Serialize` and `Deserialize` to
//! allow them to be re-used for higher-layer RPC protocols, such as
//! the HTTP server used for the `emberweb` UI client.

pub mod contacts;
pub mod error;
pub mod messages;
pub mod services;
pub mod users;

#[cfg(feature = "rpc")]
pub mod rpc;
