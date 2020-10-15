//! Low-level binary payload messages for qaul.net
//! 
//! A set of utilities to dispatch and receive binary payload messages
//! from a remote user on the network.  Higher-level message
//! abstractions are available as external services.  Use this type if
//! none of the existing message variants apply to your payload.
//!
//! A message is always properly framed and checked for data and
//! cryptographic signature integrity.  When sending a message to a
//! set of individual users (meaning not setting [`Mode::Flood`]), it is
//! also encrypted.
//!
//! [`Mode::Flood`]: ./enum.Mode.html#variant.Flood
//!
//! Try avoid sending massive payloads to a set of recipients because
//! the routing layer won't be able to deduplicate frames that are
//! encrypted.  A filesharing service is available to make initiating
//! lazy data pulls for your service.

use crate::error::{Error, Result};
use alexandria_tags::{Tag, TagSet};
use ratman_identity::Identity;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A reference to an internally stored message object
pub type MsgRef = Arc<Message>;

/// Length of an `MsgId`, for converting to and from arrays
pub const ID_LEN: usize = 16;

/// A unique, randomly generated message ID
pub type MsgId = Identity;

#[doc(hidden)]
pub const TAG_FLOOD: &'static str = "libqaul._int.flood";
#[doc(hidden)]
pub const TAG_UNREAD: &'static str = "libqaul._int.unread";
#[doc(hidden)]
pub const TAG_SENDER: &'static str = "libqaul._int.sender";
#[doc(hidden)]
pub const TAG_SERVICE: &'static str = "libqaul._int.service";

/// Signature trust level of an incoming `Message`
///
/// The three variants encode `trusted`, `unverified` and `invalid`,
/// according to signature verification of the internal keystore.
///
/// The `SigTrust::ok` convenience function can be used to reject
/// non-verifiable (unknown or bad) `Message` signatures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigTrust {
    /// A verified signature by a known contact
    Trusted,
    /// An unverified signature by a known contact
    /// (pubkey not available!)
    Unverified,
    /// A fraudulent signature
    Invalid,
}

impl SigTrust {
    pub fn ok(&self) -> Result<()> {
        match self {
            Self::Trusted => Ok(()),
            Self::Unverified => Err(Error::NoSign),
            Self::Invalid => Err(Error::BadSign),
        }
    }
}

/// Specify the way that a message gets dispatched
///
/// This information is only needed during transmission, because the
/// message should later be associated with some other metadata
/// provided by your service (or just the message ID).
///
/// When sending a flooded message, it becomes publicly accessible for
/// everybody on this node, and will most likely be stored in plain
/// text on receiving nodes across the network.  Be aware of this!
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    /// Send a message to everybody
    Flood,
    /// Address only a single identity
    Std(Identity),
}

impl Mode {
    pub fn id(&self) -> Option<Identity> {
        match self {
            Self::Std(id) => Some(*id),
            Self::Flood => None,
        }
    }
}

impl From<Identity> for Mode {
    fn from(id: Identity) -> Self {
        Self::Std(id)
    }
}

/// Specify the id type for a message dispatch
///
/// Because libqaul doesn't implement recipient groups it's up to a
/// service to create useful categorisations for groups of users.
/// This means that a service might send the same message to different
/// users, that are then receiving technically different messages.
/// This can cause all sorts of issues for services because now the
/// database is keeping track of a message many times (for each user
/// it was sent to).
///
/// This is what this type aims to circumvent: a message id can be
/// randomised during delivery, or fixed as a group to ensure that a
/// set of messages are all assigned the same Id.
///
/// **This comes with some caveats:** when inserting into the
/// database, the message Id will already exist, and so further
/// messages will not be stored.  If you are using the grouped
/// constraint on an unequal message set (meaning that payloads
/// differ), this will result in data loss!
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IdType {
    /// A unique message ID will be generated on dispatch
    Unique,
    /// Create a grouped message ID constraint
    Grouped(MsgId),
}

impl IdType {
    pub fn consume(self) -> MsgId {
        match self {
            Self::Unique => MsgId::random(),
            Self::Grouped(id) => id,
        }
    }

    /// Create an ID type that is constrained for a group
    pub fn group(id: MsgId) -> Self {
        Self::Grouped(id)
    }

    /// Create a new message group with a random Id
    pub fn create_group() -> Self {
        Self::Grouped(MsgId::random())
    }

    /// Create a new message ID for every message dispatched
    pub fn unique() -> Self {
        Self::Unique
    }
}

/// A query interface for the local message store
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MsgQuery {
    pub(crate) id: Option<MsgId>,
    pub(crate) sender: Option<Identity>,
    pub(crate) tags: TagSet,
    pub(crate) skip: usize,
}

impl MsgQuery {
    /// Create a new, empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// An override query, that only searches for a specific Id
    ///
    /// Ignores all other values passed into the query
    pub fn id(id: MsgId) -> Self {
        let id = Some(id);
        Self { id, ..Self::new() }
    }

    /// Query for messages by a specific sender
    pub fn sender(self, sender: Identity) -> Self {
        Self {
            sender: Some(sender),
            ..self
        }
    }

    /// Add a tag to the query that must be present
    ///
    /// Tag queries aim to be a subset in matching messages, which
    /// means that more tags can exist for a message, but all provided
    /// tags must be present.
    pub fn tag(mut self, t: Tag) -> Self {
        self.tags.insert(t);
        self
    }

    /// A convenience function for addding the "unread" tag
    pub fn unread(self) -> Self {
        self.tag(Tag::empty(TAG_UNREAD))
    }
}

/// A multi-purpose service Message
///
/// While this representation is quite "low level", i.e. forces a user
/// to deal with payload encoding themselves and provides no
/// functionality for async payloads (via filesharing, or similar), it
/// is quite a high level abstraction considering the data that needs
/// to be sent over the network in order for it to reach it's
/// recipient.
///
/// This type is both returned by `listen`, `poll`, as well as
/// specific message `queries`
///
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Message {
    /// A unique message ID
    pub id: MsgId,
    /// The sender identity
    pub sender: Identity,
    /// The embedded service associator
    pub associator: String,
    /// A tag store for persistent message metadata
    pub tags: TagSet,
    /// A raw byte `Message` payload
    pub payload: Vec<u8>,
}
