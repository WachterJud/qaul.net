//! RPC related error handling

pub type RpcResult<T> = Result<T, RpcError>;

/// A set of errors that occur when connecting to services
#[derive(Debug)]
pub enum RpcError {
    /// No such service was found by the broker
    NoSuchService,
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
    ConnectionFault,
    /// Encoding or decoding a payload failed
    EncoderFault,
    /// Any other failure with it's error message string
    Other(String),
}


impl From<std::io::Error> for RpcError {
    fn from(_: std::io::Error) -> Self {
        Self::ConnectionFault
    }
}

impl From<capnp::Error>    for RpcError {
    fn from(_: capnp::Error) -> Self {
        Self::EncoderFault
    }
}
