//! Service inteface utilities

use crate::{
    error::{Error, Result},
    messages::MsgRef,
    users::UserAuth,
};
use alexandria::Library;
use async_std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod store;

pub use libqaul_types::services::{MetadataMap, Service, ServiceEvent};

pub(crate) use self::store::MetadataStore;

pub(crate) type Listener = Arc<dyn Fn(ServiceEvent) + Send + Sync>;

/// A registered service, with a pre-made poll setup and listeners

/// Keeps track of registered services and their callbacks
#[derive(Clone)]
pub(crate) struct ServiceRegistry {
    notify: Arc<RwLock<BTreeMap<String, Listener>>>,
    store: MetadataStore,
}

impl ServiceRegistry {
    pub(crate) fn new(library: Arc<Library>) -> Self {
        Self {
            notify: Arc::new(RwLock::new(BTreeMap::new())),
            store: MetadataStore::new(library),
        }
    }

    /// Get access to the inner service store
    pub(crate) fn store(&self) -> &MetadataStore {
        &self.store
    }

    /// Send an event to all services that a user has come online
    pub(crate) async fn open_user(&self, auth: &UserAuth) {
        self.notify
            .read()
            .await
            .iter()
            .for_each(|(_, fun)| fun(ServiceEvent::Open(auth.clone())));
    }

    /// Send an event to all services that a user has come online
    pub(crate) async fn close_user(&self, auth: &UserAuth) {
        self.notify
            .read()
            .await
            .iter()
            .for_each(|(_, fun)| fun(ServiceEvent::Close(auth.clone())));
    }

    pub(crate) async fn register<F: 'static>(&self, name: String, listen: F) -> Result<()>
    where
        F: Fn(ServiceEvent) + Send + Sync,
    {
        let mut map = self.notify.write().await;
        if map.contains_key(&name) {
            Err(Error::ServiceExists)
        } else {
            map.insert(name, Arc::new(listen));
            Ok(())
        }
    }

    /// Check if a service was registered before
    pub(crate) async fn check(&self, name: &String) -> Result<()> {
        self.notify
            .read()
            .await
            .get(name)
            .map_or(Err(Error::NoService), |_| Ok(()))
    }

    pub(crate) async fn unregister(&self, name: String) -> Result<()> {
        let mut map = self.notify.write().await;
        map.remove(&name).map_or(Err(Error::NoService), |_| Ok(()))
    }
}
