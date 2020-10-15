//! Store for user profiles

use crate::{
    error::{Error, Result},
    qaul::Identity,
    security::{KeyId, Keypair},
    store::{FromRecord, KeyWrap, UserProfileExt},
    users::{UserProfile, UserUpdate},
};
use alexandria::{
    query::{Query, QueryResult},
    utils::{Id, Path, Tag, TagSet},
    Library, Session, GLOBAL,
};

use std::{collections::BTreeSet, sync::Arc};

pub(crate) const TAG_PROFILE: &'static str = "libqaul.user.profile";
pub(crate) const TAG_LOCAL: &'static str = "libqaul.user.local";

fn key_path(id: Id) -> Path {
    Path::from(format!("/users/keys:{}", id))
}

fn profile_path(id: Id) -> Path {
    Path::from(format!("/users:{}", id))
}

/// A type wrapper around the alexandria storage library
// FIXME: make this not clone, explicitly Arc or UserStoreRef
#[derive(Clone)]
pub(crate) struct UserStore {
    inner: Arc<Library>,
}

impl UserStore {
    /// Create a new type abstraction over an existing Alexandria lib
    pub(crate) fn new(inner: Arc<Library>) -> Self {
        Self { inner }
    }

    /// Create a new local user
    pub(crate) async fn create_local(&self, keyid: KeyId, pw: &str) {
        let KeyId { id, keypair } = keyid;
        self.inner.sessions().create(id, pw).await.unwrap();
        let wrapped = KeyWrap(keypair);

        // Store the key
        self.inner
            .insert(
                Session::Id(id),
                key_path(id),
                TagSet::empty(),
                wrapped.make_diff(),
            )
            .await
            .unwrap();

        self.insert_profile(id, vec![Tag::empty(TAG_PROFILE), Tag::empty(TAG_LOCAL)])
            .await;
    }

    /// Add an empty user profile for an id
    pub(crate) async fn insert_profile<T: Into<TagSet>>(&self, id: Identity, tags: T) {
        let profile = UserProfile::new(id);
        self.inner
            .batch(GLOBAL, profile_path(id), tags, profile.init_diff())
            .await
            .unwrap();
    }

    /// Delete the key and profile for a local user
    pub(crate) async fn delete_local(&self, id: Identity) {
        self.get(id).await.unwrap();

        self.inner
            .delete(Session::Id(id), key_path(id))
            .await
            .unwrap();

        self.inner.delete(GLOBAL, profile_path(id)).await.unwrap();
    }

    /// Modify a single user inside the store in-place
    pub(crate) async fn modify(&self, id: Identity, modifier: UserUpdate) -> Result<()> {
        let curr = self.get(id).await?;
        let diff = curr.gen_diff(modifier);
        self.inner
            .update(GLOBAL, profile_path(id), diff)
            .await
            .unwrap();
        Ok(())
    }

    /// Don't call this on non-local users please
    pub(crate) async fn get_key(&self, id: Identity) -> Keypair {
        match self
            .inner
            .query(Session::Id(id), Query::Path(key_path(id)))
            .await
        {
            Ok(QueryResult::Single(rec)) => KeyWrap::from_rec(rec).0,
            _ => panic!("Local encryption key not known!"),
        }
    }

    /// Get a specific user profile
    pub(crate) async fn get(&self, id: Identity) -> Result<UserProfile> {
        match self
            .inner
            .query(GLOBAL, Query::Path(profile_path(id)))
            .await
        {
            Ok(QueryResult::Single(rec)) => Ok(UserProfile::from_rec(rec)),
            Err(_) => Err(Error::NoUser),
            _ => unimplemented!(),
        }
    }

    /// Get all locally available users
    pub(crate) async fn all_local(&self) -> Vec<UserProfile> {
        match self
            .inner
            .query(
                GLOBAL,
                Query::tags().subset(vec![Tag::empty(TAG_PROFILE), Tag::empty(TAG_LOCAL)]),
            )
            .await
            .unwrap()
        {
            QueryResult::Many(vec) => vec.into_iter().map(|rec| UserProfile::from_rec(rec)).collect(),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn known_remote(&self) -> BTreeSet<Identity> {
        self.all_remote().await.into_iter().map(|p| p.id).collect()
    }

    /// Get all remote users this device knows about
    #[allow(unused)]
    pub(crate) async fn all_remote(&self) -> Vec<UserProfile> {
        match self
            .inner
            .query(GLOBAL, Query::tags().equals(Tag::empty(TAG_PROFILE)))
            .await
            .unwrap()
        {
            QueryResult::Many(vec) => vec.into_iter().map(|rec| UserProfile::from_rec(rec)).collect(),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod harness {
    use crate::{users::UserStore, Identity};
    use alexandria::utils::Tag;

    pub(super) fn setup() -> UserStore {
        use alexandria::Builder;
        let dir = tempfile::tempdir().unwrap();
        let lib = Builder::new().offset(dir.path()).build().unwrap();
        UserStore::new(lib)
    }

    /// Insert a random user into the store and return the Id
    pub(super) fn insert_random(store: &UserStore) -> Identity {
        async_std::task::block_on(async {
            use crate::security::Sec;
            let keyid = Sec::new().generate().await;
            let id = keyid.id;
            store.create_local(keyid, "car horse battery staple").await;
            id
        })
    }

    /// Insert a random user into the store and return the Id
    #[allow(unused)]
    pub(super) fn insert_random_remote(store: &UserStore) -> Identity {
        async_std::task::block_on(async {
            let id = Identity::random();
            store.insert_profile(id, vec![Tag::empty("profile")]).await;
            id
        })
    }
}

#[test]
fn create_user() {
    let store = harness::setup();
    harness::insert_random(&store);
}

#[async_std::test]
async fn create_and_get_key() {
    let store = harness::setup();
    let id = harness::insert_random(&store);
    store.get_key(id).await;
}

#[async_std::test]
async fn create_and_get_user() {
    let store = harness::setup();
    let id = harness::insert_random(&store);
    let user = store.get(id).await.unwrap();
    assert_eq!(user.id, id);
}

#[async_std::test]
async fn update_user() {
    let store = harness::setup();
    let id = harness::insert_random(&store);

    let update = UserUpdate::DisplayName(Some("spacekookie".into()));
    store.modify(id, update).await.unwrap();

    let after = store.get(id).await.unwrap();
    assert_eq!(after.display_name, Some("spacekookie".into()));
}

#[async_std::test]
async fn delete_user() {
    let store = harness::setup();
    let id = harness::insert_random(&store);
    assert_eq!(store.all_local().await.len(), 1);

    store.delete_local(id).await;
    assert_eq!(store.all_local().await.len(), 0);
}

#[async_std::test]
async fn create_local_and_query() {
    let store = harness::setup();
    let _ = harness::insert_random(&store);
    assert_eq!(store.all_local().await.len(), 1);
}
