//! A common abstraction over several network backplanes

mod auth;
mod crypto;
mod storage;

mod users;
pub use users::{ContactBook, ContactUpdate, LocalContactData, User, UserData, UserUpdate};

// This module defines the libqaul service API
mod api;
pub use api::*;

pub use identity::Identity;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// Primary context structure for `libqaul`
///
/// Handles user state, secret storage, network state,
/// I/O and services. Check `api` for the extended
/// service API
///
/// ## Bootstrapping
///
/// Starting an instance of `libqaul` requires several steps.
/// For one, it needs to be initialised with a valid config
/// for the routing-layer (`RATMAN`). This requires choosing
/// of network backends and client configuration.
///
/// Secondly, `libqaul` by itself does very little, except handle
/// service requests. The service API exposes various workloads
/// available, but the consuming services also need to be configured,
/// externally to `libqaul` and this instance.
///
/// A bootstrapping procedure should thus look as follows:
///
/// 1. RATMAN + netmod initialisation
/// 2. `libqaul` startup (this struct, call `init()`)
/// 3. Initialise services with a `libqaul` instance reference
/// 4. Your application is now ready for use
#[derive(Clone)]
pub struct Qaul {
    users: Arc<Mutex<BTreeMap<Identity, User>>>,
    auth: Arc<Mutex<BTreeMap<Identity, String>>>,
    keys: Arc<Mutex<BTreeMap<String, Identity>>>,
    contacts: Arc<Mutex<BTreeMap<Identity, ContactBook>>>,
    services: Arc<Mutex<BTreeMap<String, Arc<Box<dyn Service + 'static>>>>>
}

impl Qaul {
    pub fn start() -> Self {
        Self {
            users: Arc::new(Mutex::new(BTreeMap::new())),
            auth: Arc::new(Mutex::new(BTreeMap::new())),
            keys: Arc::new(Mutex::new(BTreeMap::new())),
            contacts: Arc::new(Mutex::new(BTreeMap::new())),
            services: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}
