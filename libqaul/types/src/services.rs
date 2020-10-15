//! Application service management and utilities
//!
//! An application using libqaul is called a "service". qaul.net (the
//! application) is simply a collection of services that expose a
//! common UI for users to interact with each other.  A service
//! doesn't need to be user-facing, or have a UI.
//!
//! Via the [`qrpc`] message bus it is possible for arbitrary
//! processes to interact with other services, and libqaul instances.
//! Because libqaul implements encrypted at-rest storage, this
//! mechanism is exposed to services via this API.  This way your
//! application can't accidentally leak user metadata.
//!
//! [`qrpc`]: https://docs.qaul.net/api/qrpc-sdk/index.html

use crate::users::UserAuth;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

/// An arbitrary map of metadata that can be stored by a service
///
/// Data is stored per service/per user and is tagged with search
/// tags.  This structure (and API) can be used to store service
/// related data on a device that will be encrypted and can be loaded
/// on reboot, meaning that your service doesn't have to worry about
/// storing things securely on different platforms.
///
/// `MetadataMap` has a builder API that makes constructing initial
/// maps easier than just providing an already initialised BTreeMap.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MetadataMap {
    name: String,
    map: BTreeMap<String, Vec<u8>>,
}

impl MetadataMap {
    /// Creates a new, empty metadata map
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            map: Default::default(),
        }
    }

    /// Create a metadata map from a name and initialised map construct
    ///
    /// ```
    /// # use libqaul::services::MetadataMap;
    /// MetadataMap::from("numbers", vec![("fav", vec![1, 2, 3, 4])]);
    /// ```
    ///
    /// Because from takes `IntoIterator`, you can also initialise
    /// your map in-place:
    ///
    /// ```
    /// # use libqaul::services::MetadataMap;
    /// MetadataMap::from("numbers", vec![
    ///     ("fav", vec![1, 2, 3, 4]),
    ///     ("prime", vec![1, 3, 5, 7, 11]),
    ///     ("acab", vec![13, 12]),
    /// ]);
    /// ```
    pub fn from<S, K, M, V>(name: S, map: M) -> Self
    where
        S: Into<String>,
        K: Into<String>,
        M: IntoIterator<Item = (K, V)>,
        V: IntoIterator<Item = u8>,
    {
        let name = name.into();
        let map = map
            .into_iter()
            .map(|(k, v)| (k.into(), v.into_iter().collect()))
            .collect();
        Self { name, map }
    }

    /// Return this entries name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Add (and override) a key-value map and return the modified map
    pub fn add<K, V>(mut self, k: K, v: V) -> Self
    where
        K: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.map.insert(k.into(), v.into());
        self
    }

    /// Delete a key and return the modified map
    pub fn delete<K: Into<String>>(mut self, k: K) -> Self {
        self.map.remove(&k.into());
        self
    }
}

impl Deref for MetadataMap {
    type Target = BTreeMap<String, Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for MetadataMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// Represents a service using libqaul
///
/// Via this type it's possible to either perform actions as a
/// particular survice, or none, which means that all service's events
/// become available.  While this is probably not desirable (and
/// should be turned off) in most situations, this way a user-level
/// service can do very powerful things with the "raw" netork traffic
/// of a qaul network.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Service {
    /// Get access to all service's events
    // One of the three most common passwords, you know?
    God,
    /// Service by domain qualified name (e.g. `net.qaul.chat`)
    Name(String),
}

impl<T> From<T> for Service
where
    T: Into<String>,
{
    fn from(t: T) -> Self {
        Self::Name(t.into())
    }
}

/// Event type that can be sent to services to react to state changes
pub enum ServiceEvent {
    Open(UserAuth),
    Close(UserAuth),
}
