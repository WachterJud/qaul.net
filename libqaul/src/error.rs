//! Error and Result handling
//!
//! `libqaul` spans over a huge abstraction surface, from drivers all
//! the way to "business logic" functions in the service API (see
//! `api` module). This makes communicating errors challenging at
//! times. Generally, no lower layer `Error` objects are wrapped here,
//! to avoid introducing new dependencies in service code.
//!
//! Instead, `Error` attempts to provide a comprehensive set of
//! failures modes, that can be returned to communicate a failure,
//! that then needs tobe interpreted and addressed by an implementing
//! application. This way, it becomes easier for _your_ service to
//! wrap errors, or to enumerate them more easily.
//!
//! On an `Error` enum, it is also possible to call `description()` to
//! get a plain text error description of what went wrong, and what it
//! probably means. These are meant to simplify front-end development
//! and avoid having applications return arbitrary codes. You can also
//! set `QAUL_LANG=ar` (or others) as an environment variable to get
//! translations of these messages, with `en` being the fallback.

use ratman::Error as RatError;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    result::Result as StdResult,
};

pub use libqaul_types::error::{Error, Result};

pub(crate) fn bincode() -> Error {
    Error::BadSerialise
}

pub(crate) fn ratman(e: RatError) -> Error {
    Error::NetworkFault
}
