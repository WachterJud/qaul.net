//! Service related error handling

pub type Result<T> = std::result::Result<T, ServiceError>;

/// A set of errors that occur when connecting to services
#[derive(Debug)]
pub enum ServiceError {
    /// No such service was found by the broker
    NoSuchService,
    /// The service didn't reply within the timeout time
    ///
    /// This may indicate that the requested service has crashed, is
    /// dealing with backpressure, or the broker is quietly dropping
    /// requests.
    ServiceBusy,
    /// Tried connecting to a service that's already connected
    AlreadyConnected,
    /// Failed to perform action that requires a connection
    NotConnected,
    /// Any other failure with it's error message string
    Other(String),
}
