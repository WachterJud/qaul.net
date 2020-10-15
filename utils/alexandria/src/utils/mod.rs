//! A set of utilities to work with an alexandria library
//!
//! Most of these are rather fundamental, and are clustered here to
//! simplify imports.

/// Primary identifier type for records and users
///
/// Internally an Id is represented as the size of a word on the
/// platform that alexandria was compiled for.  Each Id is unique and
/// random (via `Id::random()`).
pub use id::Identity as Id;

mod diff;
pub use diff::{Diff, DiffSeg};
pub(crate) use diff::{DiffExt, DiffResult};

mod path;
pub use path::Path;

pub use alexandria_tags::{Tag, TagSet};
