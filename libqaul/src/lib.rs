//! This is the library that sits at the heart of
//! [qaul.net](https://qaul.net).
//!
//! Fundamentally, it handles three types of interactions:
//!
//! - Initialised hardware messaging
//! - Service messaging and service hosting
//! - Internal storage, parsing and encryption
//!
//! The two things provided by libqaul, that make it useful to other
//! applications are `Qaul`, the primary data struct and API holder,
//! plus the "service API" (implemented on `Qaul`) which gives a
//! developer access to a qaul network.
//!
//! The initilisation order for the libqaul stack is reverted.
//!
//! 1. Hardware modules (`netmod`) for the appropriate platform
//! 2. `RATMAN` routing core, binding against available network
//!    interfaces
//! 3. (Optional) Platform specific storage shims for `Alexandria`
//! 4. `Qaul` internals, which sets up storage, encryption and
//!    user stores
//! 5. API shims such as the `http-api` which exposes the service
//!    API on a json:api schema
//! 6. UI threads: either initialise the qaul.net web-frontend or
//!    your own application stack
//!
//! `libqaul` handles user registration, sign-in and out, messaging,
//! file-sharing, both encrypted and public communication, voice
//! calls, as well as service hooks that mean that your applications
//! can communicate with the existing services, and other instances
//! running across a qaul network.
//!
//! Additionally to providing the entire application stack, `libqaul`
//! can also tunnel to other `libqaul` instances, depending on the
//! platform.  This means that your application might be shipping an
//! entire copy of `libqaul`, but doesn't have to be the network entry
//! point. This initialisation option is available before starting
//! network bindings.

#![doc(html_favicon_url = "https://qaul.net/favicon.ico")]
#![doc(html_logo_url = "https://qaul.net/img/qaul_icon-128.png")]

// Internal modules
mod auth;
mod crypto;
mod discover;
mod security;
mod store;
mod utils;

// Exposed API modules
pub mod api;
pub mod contacts;
pub mod error;
pub mod helpers;
pub mod messages;
pub mod services;
pub mod users;

#[cfg(feature = "ffi-java")]
pub mod ffi;

#[cfg(feature = "rpc")]
pub mod rpc;

// Core state should be in the root
mod qaul;
pub use qaul::{Identity, Qaul, QaulRef};
