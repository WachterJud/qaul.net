//! Network user contact metadata and utilities
//!
//! Users' contact books exist as:
//!
//! - lists of users they know about, by identity, plus
//! - local-only information about those users, like personal nicknames

// Public exports
pub use libqaul_types::contacts::{ContactEntry, ContactQuery};

use crate::{
    error::{Error, Result},
    Identity,
};

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// A collection of contacts associated with their local-only data.
pub(crate) type ContactList = BTreeMap<Identity, ContactEntry>;

/// Wraps around user-local contact books
#[derive(Clone, Debug)]
pub(crate) struct ContactStore {
    inner: Arc<Mutex<BTreeMap<Identity, ContactList>>>,
}

impl ContactStore {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Modify a users personal contact entry via a callback
    ///
    /// `id` in this case is the current session user, `user` is the
    /// contact entry they want to modify. **If none previously
    /// existed, a fresh one will be created.**
    pub(crate) fn modify<F>(&self, id: &Identity, user: &Identity, modify: F)
    where
        F: FnOnce(&mut ContactEntry),
    {
        let mut inner = self.inner.lock().expect("Failed to lock ContactStore");
        let contact = inner
            .entry(*id)
            .or_insert(Default::default())
            .entry(*user)
            .or_insert(Default::default());
        modify(contact);
    }

    /// Query a user's contact book data
    ///
    /// If this user hasn't yet specified any contact entries, then
    /// just return an empty list instead.
    pub(crate) fn query(&self, id: &Identity, query: ContactQuery) -> Result<Vec<Identity>> {
        let mut inner = self.inner.lock().expect("Failed to lock ContactStore");
        Ok(inner
            .entry(*id)
            .or_insert(Default::default())
            .iter()
            .filter(|(_, con)| match &query {
                ContactQuery::Nick(_) if con.nick.is_none() => false,
                ContactQuery::Nick(nick) => con.nick.as_ref().unwrap().contains(nick),
                ContactQuery::Trust { val, fuz } => {
                    con.trust + fuz < *val || con.trust - fuz > *val
                }
                ContactQuery::Met(met) => con.met == *met,
                ContactQuery::Location(_) if con.location.is_none() => false,
                ContactQuery::Location(loc) => con.location.as_ref().unwrap().contains(loc),
                ContactQuery::Notes(_) if con.notes.is_none() => false,
                ContactQuery::Notes(notes) => con.notes.as_ref().unwrap().contains(notes),
            })
            .map(|(id, _)| id.clone())
            .collect())
    }

    pub(crate) fn get(&self, id: &Identity, contact: &Identity) -> Result<ContactEntry> {
        let inner = self.inner.lock().expect("Failed to lock ContactStore");
        inner
            .get(id)
            .map_or(Err(Error::NoUser), |x| Ok(x))?
            .get(contact)
            .map_or(Err(Error::NoContact), |x| Ok(x.clone()))
    }

    #[allow(unused)]
    pub(crate) fn get_all(&self, id: &Identity) -> Result<Vec<Identity>> {
        let inner = self.inner.lock().expect("Failed to lock ContactStore");
        Ok(inner
            .get(id)
            .map_or(Err(Error::NoUser), |x| Ok(x))?
            .iter()
            .map(|(id, _)| id.clone())
            .collect())
    }
}
