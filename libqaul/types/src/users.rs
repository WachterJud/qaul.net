//! Network user and session management
//!
//! `libqaul` is an abstraction over a distributed network of users,
//! meaning that it is impossible to tell the underlying device
//! configuration.  When connecting to other applications on the
//! network, this is always done between *users*. 


use ratman_identity::Identity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A random authentication token
pub type Token = String;

/// Wrapper to encode `User` authentication state
///
/// This structure can be aquired by challenging an authentication
/// endpoint, such as `User::login` to yield a token. If a session for
/// this `Identity` already exists, it will be re-used.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserAuth(pub Identity, pub Token);

/// A complete user profile with ID and metadata
///
/// The user profile itself makes no destinction between local, remote
/// or self users (the latter being the currently active user in a
/// session)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    /// A user's network (node) ID
    pub id: Identity,
    /// A human readable display-name (like @foobar)
    #[serde(default)]
    pub display_name: Option<String>,
    /// A human's preferred call-sign ("Friends call me foo")
    #[serde(default)]
    pub real_name: Option<String>,
    /// A key-value list of things the user deems interesting about
    /// themselves. This could be stuff like "gender", "preferred
    /// languages" or whatever.
    #[serde(default)]
    pub bio: BTreeMap<String, String>,
    /// The set of services this user runs (should never be empty!)
    #[serde(default)]
    pub services: BTreeSet<String>,
    /// A users profile picture (some people like selfies)
    #[serde(default)]
    pub avatar: Option<Vec<u8>>,
}

/// All the ways a UserData can change, as individual events.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UserUpdate {
    /// Set or blank the User's display name.
    DisplayName(Option<String>),
    /// Set or blank the User's real name.
    RealName(Option<String>),
    /// Add or update a biography line with the given key to the given value.
    SetBioLine(String, String),
    /// Remove a biography line with the given key, or do nothing if it does not exist.
    RemoveBioLine(String),
    /// Add a service with the given name.
    AddService(String),
    /// Remove the service with the given name, or do nothing if it does not exist.
    RemoveService(String),
    /// Set or blank the User's avatar.
    AvatarData(Option<Vec<u8>>),
}
