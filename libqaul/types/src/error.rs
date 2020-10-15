//! Error and Result handling
//!
//! `libqaul` spans over a large abstraction surface, from drivers all
//! the way to "business logic" functions in the service API. This
//! makes communicating errors challenging at times. Generally, no
//! lower layer `Error` objects are wrapped here, to avoid introducing
//! new dependencies in service code.
//!
//! Instead, `Error` attempts to provide a comprehensive set of
//! failures modes, that can be returned to communicate a failure,
//! that then needs tobe interpreted and addressed by an implementing
//! application. This way, it becomes easier for _your_ service to
//! wrap errors, or to enumerate them more easily.
//!
//! On an `Error` enum, it is also possible to call `help()` to
//! get a plain text error description of what went wrong, and what it
//! probably means. These are meant to simplify front-end development
//! and avoid having applications return arbitrary codes.

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    result::Result as StdResult,
};

/// `libqaul` specific Result with embedded Error
///
/// The returned `Error` can sometimes be considered non-fatal. Check
/// the `Error` documentation for the specific returned variant to
/// see, what level of fatality it should be interpreted as. Crashing
/// on every returned `Err(_)` however is a bad idea.
pub type Result<T> = StdResult<T, Error>;

/// `libqaul` service API error states
///
/// All errors that can occur in interaction with the API are encoded
/// as variants on this enum. In most cases, no additional metadata is
/// provided and needs to be inferred from whatever context or
/// function call emitted the error. Check the variant doc comments
/// for a broad overview, as well as detailed usage instructions.
///
/// ## A note on language
///
/// Most variants of this enum use either an `Invalid` or `No`
/// prefix. Invalid data is data that was either not expected or badly
/// formatted. `No` in this case takes the place of `Unknown`, meaning
/// that a query could not be fulfilled.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    /// Not authorised to perform this action
    NotAuthorised,
    /// The desired user was not known
    NoUser,
    /// The provided contact already exists
    ContactExists,
    /// The desired contact does not exist
    NoContact,
    /// Invalid search query
    InvalidQuery,
    /// No data was returned for the provided query
    NoData,
    /// Invalid payload (probably too big)
    InvalidPayload,
    /// A function callback timed out
    CallbackTimeout,
    /// Signature with an unknown public key
    NoSign,
    /// Fraudulent signature for a known public key
    BadSign,
    /// A generic networking error occured
    NetworkFault,
    /// Failed to find a route to this user
    NoRoute,
    /// Some serialisation action failed
    BadSerialise,
    /// No such service was found
    NoService,
    /// A sevice with this name already exists
    ServiceExists,
    /// Some internal components failed to communicate
    CommFault,
}

impl Error {
    pub fn help(&self) -> String {
        match std::env::var("QAUL_LANG").as_ref().map(|s| s.as_str()) {
            Ok("ar") => "حدث خطأ غير معروف",
            Ok("de") => "Ein unbekannter Fehler ist aufgetreten",
            _ => "An unknown Error occured",
        }
        .into()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::NotAuthorised => "Not authorised to perform this action",
            Self::NoUser => "The desired user was not known",
            Self::ContactExists => "The provided contact already exists",
            Self::NoContact => "The desired contact does not exist",
            Self::InvalidQuery => "Invalid search query",
            Self::NoData => "No data was returned for the provided query",
            Self::InvalidPayload => "Invalid payload (probably too big)",
            Self::CallbackTimeout => "A function callback timed out",
            Self::NoSign => "Signature with an unknown public key",
            Self::BadSign => "Fraudulent signature for a known public key",
            Self::NetworkFault => "A generic networking error occured",
            Self::NoRoute => "Failed to find a route to this user",
            Self::BadSerialise => "Some serialisation action failed",
            Self::NoService => "No such service was found",
            Self::ServiceExists => "A sevice with this name already exists",
            Self::CommFault => "Some internal components failed to communicate",
        };
        write!(f, "{}", msg)
    }
}

impl StdError for Error {}
