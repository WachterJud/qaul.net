//! # libqaul RPC structures
//!
//! This library exposes structures and serialisation utilities to
//! interact with the `libqaul` API remotely, via a simple RPC
//! protocol.  By default the `proto` feature flag is enabled that
//! also creates a serialisation/deserialisation protocol via [cap'n
//! proto](https://capnproto.org).
//!
//! The RPC protocol defined in this library makes no assumption about
//! the layering/ framing used to communicate with the libqaul core.
//! You can layer it over `libqaul-ws` for web sockets, `libqaul-ipc`
//! for a client/server socket API, or `libqaul-http` for a json http
//! api.
//!
//! In order to use this RPC crate correctly you will also have to
//! depend on the `libqaul` crate for structure, error, and return
//! type definitions.

#![allow(unused)]
#![doc(html_favicon_url = "https://qaul.net/favicon.ico")]
#![doc(html_logo_url = "https://qaul.net/img/qaul_icon-128.png")]

mod api;
pub use api::{
    contacts, messages, users, Envelope, QaulExt, QaulRpc, Request, Responder, Response,
    StreamResponder, Streamer, SubId,
};

#[cfg(feature = "chat")]
pub use api::{chat, chat::ChatExt, chat::ChatRpc};

#[cfg(feature = "voice")]
pub use api::{voice, voice::VoiceExt, voice::VoiceRpc};

pub mod json;
