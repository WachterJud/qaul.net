
//! RPC related error handling

use std::fmt::{self, Display, Formatter};

pub type RpcResult<T> = Result<T, RpcError>;

/// A set of errors that occur when connecting to services
#[derive(Debug)]
pub enum RpcError {
    /// No such service was found by the broker
    NoSuchService(String),
    /// The selected recipient didn't reply within the timeout
    ///
    /// This may indicate that the requested service has crashed, is
    /// dealing with backpressure, or the broker is quietly dropping
    /// requests.
    Timeout,
    /// Tried connecting to a service that's already connected
    AlreadyConnected,
    /// Failed to perform action that requires a connection
    NotConnected,
    /// Invalid connection: performing the last operation has failed
    ConnectionFault(String),
    /// Encoding or decoding a payload failed
    EncoderFault(String),
    /// Any other failure with it's error message string
    Other(String),
}

impl Display for RpcError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoSuchService(s) => format!("The requested service {} does not exist!", s),
                Self::Timeout => "The requested operation took too long (timeout)!".into(),
                Self::AlreadyConnected =>
                    "Tried connecting to an already connected component!".into(),
                Self::NotConnected =>
                    "Tried to perform an action that needs a component connection!".into(),
                Self::ConnectionFault(s) => format!("I/O error: {}", s),
                Self::EncoderFault(s) => format!("Encode error: {}", s),
                Self::Other(s) => format!("Unknown error: {}", s),
            }
        )
    }
}

impl From<std::io::Error> for RpcError {
    fn from(e: std::io::Error) -> Self {
        Self::ConnectionFault(e.to_string())
    }
}

impl From<capnp::Error> for RpcError {
    fn from(e: capnp::Error) -> Self {
        Self::EncoderFault(e.to_string())
    }
}

impl From<async_std::future::TimeoutError> for RpcError {
    fn from(_: async_std::future::TimeoutError) -> Self {
        Self::Timeout
    }
}
