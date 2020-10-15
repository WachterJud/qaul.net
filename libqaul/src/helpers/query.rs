//! A utility module to query messages

use crate::{
    error::{Error, Result},
    store::FromRecord,
};
use alexandria::{query::QueryIterator, record::RecordRef};
use std::marker::PhantomData;

/// The resulting set of a query operation
pub struct QueryResult<T>
where
    T: FromRecord<T>,
{
    inner: QueryIterator,
    _type: PhantomData<T>,
}

impl<T> QueryResult<T>
where
    T: FromRecord<T>,
{
    pub(crate) fn new(inner: QueryIterator) -> Self {
        Self {
            inner,
            _type: PhantomData,
        }
    }

    /// Lock the garbage collection on iteratable items
    ///
    /// By default items that are contained in this result set can be
    /// deleted by other tasks, resulting in errors when accessing
    /// data via this type.  To hold the GC from running for deleted
    /// types, you can lock the set, meaning that deleted items will
    /// only be deleted when the last locked iterator goes out of
    /// scope.
    ///
    /// This may introduce race coditions for other queries, so be
    /// aware of that!
    pub async fn lock(&self) {
        self.inner.lock().await;
    }

    /// Skip a certain number of items in the result set
    pub fn skip(&self, num: usize) {
        self.inner.skip(num);
    }

    /// Try to get a single element from the query iterator
    pub async fn next(&self) -> Result<T> {
        match self.inner.next().await {
            Ok(Some(rec)) => Ok(T::from_rec(rec)),
            _ => Err(Error::NoData),
        }
    }

    /// Take all elements from the iterator and drop invalid reads
    ///
    /// This is a semi-destructive operation because read errors will
    /// be dropped, but sometimes that is exactly what you want.
    /// Because libqaul stores messages for users separately from the
    /// "global" scope (i.e. flooded messages), a specific path query
    /// will return two paths: one will fail, the other will not.
    ///
    /// Instead of having to filter the query results manually, this
    /// function does it internally and folds into an empty vector,
    /// meaning: if both reads fail, the resolved set will be empty.
    pub async fn resolve(&self) -> Vec<T> {
        let mut vec = vec![];

        for _ in 0..self.inner.remaining() {
            if let Ok(t) = self.next().await {
                vec.push(t);
            }
        }

        vec
    }

    /// Take a number of items from the iterator to advance
    ///
    /// If no more items are present, this function will return
    /// `Err()`.  When less than the requested number of items are
    /// present, it will return `Ok()` but with all remaining items.
    pub async fn take(&self, num: usize) -> Result<Vec<T>> {
        if self.inner.remaining() == 0 {
            return Err(Error::NoData);
        }

        let mut vec = Vec::with_capacity(num);
        let mut ctr = 0;
        while let Ok(Some(rec)) = self.inner.next().await {
            vec.push(T::from_rec(rec));

            ctr += 1;
            if ctr > num && break {}
        }

        Ok(vec)
    }

    /// Take elements from the iterator until a read fails
    ///
    /// Once a read fails that does't mean that there's no more data
    /// left, just that a read was unsuccessful.  If you want to
    /// quietly drop errors, use `resolve()` instead!
    pub async fn all(&self) -> Result<Vec<T>> {
        let mut vec = vec![];
        while let Ok(Some(rec)) = self.inner.next().await {
            vec.push(T::from_rec(rec));
        }
        Ok(vec)
    }
}
