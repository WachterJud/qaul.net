//! A qrpc type and api wrapper for libqaul
//!
//! The API surface is exposed via the `QaulRpc` type, while data
//! types are exposed via the `libqaul-types` crate (re-exported from
//! this crate via [`types`]).
//!
//! Check the qrpc-sdk documentation to learn how to use this crate.

/// A qrpc wrapper for libqaul
///
/// This component exposes a public API surface to mirror the libqaul
/// crate.  This means that other clients on the qrpc bus can include
/// this surface to get access to all libqaul functions, thate are
/// transparently mapped to the underlying libqaul instance
/// potentially running in a different process.
pub struct QaulRpc {
    
}
