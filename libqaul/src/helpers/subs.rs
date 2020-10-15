use crate::{helpers::Tagged, store::FromRecord, Identity};
use alexandria::{
    query::{Query, QueryResult, Subscription as Sub},
    record::RecordRef,
    Library, Session,
};
use async_std::sync::Arc;
use std::marker::PhantomData;
use tracing::trace;

/// A unique, randomly generated subscriber ID
pub type SubId = Identity;

/// A generic subscription which can stream data from libqaul
pub struct Subscription<T>
where
    T: FromRecord<T> + Tagged,
{
    store: Arc<Library>,
    session: Session,
    inner: Sub,
    _none: PhantomData<T>,
}

impl<T> Subscription<T>
where
    T: FromRecord<T> + Tagged,
{
    pub(crate) fn new(store: &Arc<Library>, session: Session, inner: Sub) -> Self {
        Self {
            store: Arc::clone(store),
            session,
            inner,
            _none: PhantomData,
        }
    }

    /// Poll for the next return from the subscription
    pub async fn next(&self) -> T {
        // Because subscriptions also get notified about deletions we
        // basically internally drop objects that don't exist anymore
        // because it means there was probably a deletion
        loop {
            let path = self.inner.next().await;
            trace!("Querying new path {}", path.to_string());

            let rec = self.store.query(self.session, Query::path(path)).await;

            if let Ok(QueryResult::Single(rec)) = rec {
                break T::from_rec(rec);
            }
        }
    }
}
