use crate::{
    error::Result,
    services::MetadataMap,
    store::{FromRecord, MetadataExt},
    Identity,
};
use alexandria::{
    query::{Query, QueryResult},
    utils::{Path, Tag, TagSet},
    Library, Session,
};
use async_std::sync::Arc;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
use tracing::warn;

/// Generate the service metedata entry path
fn gen_path(serv: &String, name: &String) -> Path {
    Path::from(format!("/service/{}:{}", serv, name))
}

fn tag_service(serv: &String) -> Tag {
    Tag::new("libqaul._int.service", serv.as_bytes().to_vec())
}

const TAG_METADATA: &'static str = "libqaul._int.metadata";

/// Internal metadata store wrapper for Alexandria
#[derive(Clone)]
pub(crate) struct MetadataStore {
    inner: Arc<Library>,
}

impl MetadataStore {
    pub(crate) fn new(inner: Arc<Library>) -> Self {
        Self { inner }
    }

    pub(crate) async fn save(
        &self,
        user: Identity,
        serv: String,
        data: MetadataMap,
        mut tags: TagSet,
    ) -> Result<()> {
        let k = data.name().clone();
        let sess = Session::Id(user);

        // Generate diffs based on previous value
        let diffs = match self
            .inner
            .query(sess, Query::path(gen_path(&serv, &k)))
            .await
        {
            Ok(QueryResult::Single(rec)) => data.gen_diffset(&MetadataMap::from_rec(rec)),
            Err(_) => data.init_diff(),
            _ => unreachable!(),
        };

        // Add libqaul internal search tags
        tags.insert(tag_service(&serv));
        tags.insert(Tag::empty(TAG_METADATA));

        // Try to insert, otherwise update
        if let Err(_) = self
            .inner
            .batch(Session::Id(user), gen_path(&serv, &k), tags, diffs.clone())
            .await
        {
            for diff in diffs {
                self.inner
                    .update(Session::Id(user), gen_path(&serv, &k), diff)
                    .await
                    .unwrap();
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub(crate) async fn delete(&self, user: Identity, service: String, key: String) {
        if let Err(e) = self
            .inner
            .delete(Session::Id(user), gen_path(&service, &key))
            .await
        {
            warn!("An error occured while deleting metadata: {}", e);
        }
    }

    pub(crate) async fn query(
        &self,
        user: Identity,
        serv: String,
        mut tags: TagSet,
    ) -> Vec<MetadataMap> {
        let sess = Session::Id(user);

        // Add libqaul internal search tags
        tags.insert(tag_service(&serv));
        tags.insert(Tag::empty(TAG_METADATA));

        match self.inner.query(sess, Query::tags().subset(tags)).await {
            Ok(QueryResult::Single(rec)) => vec![MetadataMap::from_rec(rec)],
            Ok(QueryResult::Many(vec)) => vec
                .into_iter()
                .map(|rec| MetadataMap::from_rec(rec))
                .collect(),
            Err(_) => vec![],
        }
    }
}
