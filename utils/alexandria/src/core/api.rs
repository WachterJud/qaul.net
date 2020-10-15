use crate::{
    core::{Session, SessionsApi},
    delta::{DeltaBuilder, DeltaType},
    error::Result,
    meta::{tags::TagCache, users::UserTable},
    query::{Query, QueryIterator, QueryResult, SetQuery, SubHub, Subscription},
    store::Store,
    utils::{Diff, Id, Path, TagSet},
};
use async_std::sync::{Arc, RwLock};
use std::fmt::Debug;
use tracing::info;

/// In-memory representation of an alexandria database
///
/// Refer to [`Builder`][builder] to configure and initialise an alexandria
/// instance.
///
/// [builder]: struct.Builder.html
pub struct Library {
    // /// The main management path
    // pub(crate) root: Dirs,
    /// Table with encrypted user metadata
    pub(crate) users: RwLock<UserTable>,
    /// Cache of tag/path mappings
    pub(crate) tag_cache: RwLock<TagCache>,
    /// The main data store
    pub(crate) store: RwLock<Store>,
    /// The state handler for subscriptions
    pub(crate) subs: Arc<SubHub>,
}

impl Library {
    /// Internally called setup function
    pub(crate) fn init(self) -> Result<Self> {
        // self.root.scaffold()?;
        Ok(self)
    }

    /// Load and re-initialise a previous database session from disk
    pub fn load<'tmp, P, S>(_: P, _: S) -> Result<Self>
    where
        P: Into<&'tmp Path>,
        S: Into<String>,
    {
        unimplemented!()
    }

    /// Load the database sessions API scope
    pub fn sessions<'lib>(&'lib self) -> SessionsApi<'lib> {
        SessionsApi { inner: self }
    }

    /// Similar to `insert`, but instead operating on a batch of Diffs
    #[tracing::instrument(skip(self, data, tags), level = "info")]
    pub async fn batch<T, D>(&self, id: Session, path: Path, tags: T, data: Vec<D>) -> Result<Id>
    where
        T: Into<TagSet>,
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Insert);
        let tags = tags.into();

        let mut store = self.store.write().await;
        let rec_id = store.batch(
            &mut db,
            id,
            &path,
            tags.clone(),
            data.into_iter().map(|d| d.into()).collect(),
        )?;
        drop(store);

        let mut tc = self.tag_cache.write().await;
        tags.iter().fold(Ok(()), |res, t| {
            res.and_then(|_| tc.insert(id, path.clone(), t.clone()))
        })?;
        drop(tc);

        self.subs.queue(db.make()).await;

        info!("Batch insert succeeded");
        Ok(rec_id)
    }

    /// Insert a new record into the library and return it's ID
    ///
    /// You need to have a valid and active user session to do so, and
    /// the `path` must be unique.
    #[tracing::instrument(skip(self, data, tags), level = "info")]
    pub async fn insert<T, D>(&self, id: Session, path: Path, tags: T, data: D) -> Result<Id>
    where
        T: Into<TagSet>,
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Insert);
        let tags = tags.into();

        let mut store = self.store.write().await;
        let rec_id = store.insert(&mut db, id, &path, tags.clone(), data.into())?;
        drop(store);

        let mut tc = self.tag_cache.write().await;
        tags.iter().fold(Ok(()), |res, t| {
            res.and_then(|_| tc.insert(id, path.clone(), t.clone()))
        })?;
        drop(tc);

        self.subs.queue(db.make()).await;

        info!("Record insert succeeded");
        Ok(rec_id)
    }

    #[tracing::instrument(skip(self), level = "info")]
    pub async fn delete(&self, id: Session, path: Path) -> Result<()> {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Delete);

        let mut store = self.store.write().await;
        store.destroy(&mut db, id, &path)?;
        drop(store);

        let mut tc = self.tag_cache.write().await;
        tc.delete_path(id, path)?;
        drop(tc);

        self.subs.queue(db.make()).await;

        info!("Record delete succeeded");
        Ok(())
    }

    /// Update a record in-place
    #[tracing::instrument(skip(self, diff), level = "info")]
    pub async fn update<D>(&self, id: Session, path: Path, diff: D) -> Result<()>
    where
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Update);

        let mut store = self.store.write().await;
        store.update(&mut db, id, &path, diff.into())?;
        drop(store);

        self.subs.queue(db.make()).await;

        info!("Record update succeeded");
        Ok(())
    }

    /// Query the database with a specific query object
    ///
    /// Request data from alexandria via a `Query` object.  A query
    /// can only touch a single parameter, such as the Record Id, the
    /// path or a set query via tags.  The data returned are snapshots
    /// or records that are immutable.  If you want to make changes to
    /// them, use `update()` with a Diff instead.
    ///
    /// Also: future writes will not propagate to the copy of the
    /// Record returned from this function, because alexandria is
    /// Copy-on-Write.  You will need to query the database again in
    /// the future.
    ///
    /// ## Examples
    ///
    /// This code makes a direct query via the path of a record.  This
    /// will only return a single record if successful.
    ///
    /// ```
    /// # use alexandria::{Builder, GLOBAL, Library, error::Result, utils::{Tag, TagSet, Path}, query::Query};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build().unwrap();
    /// let path = Path::from("/msg:alice");
    /// lib.query(GLOBAL, Query::Path(path)).await;
    /// # Ok(()) }
    /// ```
    ///
    /// ### Search tags
    ///
    /// In alexandria you can tag records with extra metadata (which
    /// is also encrypted), to make queries easier and even build
    /// relationships between records in your application.  These tags
    /// are String-keyed, with an arbitrary (or no) payload and can be
    /// used to make more precise (and fast!) search queries into the
    /// database.
    ///
    /// The constraints imposed by tag queries are modelled on set
    /// theory and can be created via the [`TagQuery`][tq] helper type.
    ///
    /// [tq]: query/struct.TagQuery.html
    ///
    /// Following are a few examples for tag queries.
    ///
    /// ```
    /// # use alexandria::{GLOBAL, Builder, Library, error::Result, utils::{Tag, TagSet, Path}, query::Query};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build().unwrap();
    /// # let tag1 = Tag::new("tag1", vec![1, 3, 1, 2]);
    /// # let tag2 = Tag::new("tag2", vec![13, 12]);
    /// let tags = TagSet::from(vec![tag1, tag2]);
    /// lib.query(GLOBAL, Query::tags().subset(tags)).await;
    /// # Ok(()) }
    /// # async_std::task::block_on(async { foo().await }).unwrap();
    /// ```
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn query<S>(&self, id: S, q: Query) -> Result<QueryResult>
    where
        S: Into<Session> + Debug,
    {
        let id = id.into();
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let store = self.store.read().await;
        match q {
            Query::Path(ref path) => store.get_path(id, path).map(|rec| QueryResult::Single(rec)),
            Query::Tag(query) => {
                let tc = self.tag_cache.read().await;

                match query {
                    SetQuery::Intersect(ref tags) => tc.get_paths(id, |o| o.intersect(tags)),
                    SetQuery::Subset(ref tags) => tc.get_paths(id, |o| o.subset(tags)),
                    SetQuery::Equals(ref tags) => tc.get_paths(id, |o| o.equality(tags)),
                    SetQuery::Not(ref tags) => tc.get_paths(id, |o| o.not(tags)),
                }
                .iter()
                .map(|p| store.get_path(id, p))
                .collect::<Result<Vec<_>>>()
                .map(|vec| QueryResult::Many(vec))
            }
            _ => unimplemented!(),
        }
    }

    /// Create an iterator from a database query
    ///
    /// The primary difference between this function and `query()` is
    /// that no records are returned or loaded immediately from the
    /// database.  Instead a query is stored, sized and estimated at
    /// the time of querying and can then be stepped through.  This
    /// allows for fetching only a range of objects, limiting memory
    /// usage.
    ///
    /// Paths that are inserted after the `QueryIterator` was
    /// constructed aren't automatically added to it, because it's
    /// internal state is atomic for the time it was created.  If you
    /// want to get updates to the database as they happen, consider a
    /// `Subscription` instead.
    ///
    /// Following is an example for an iterator query, mirroring most
    /// of the `query()` usage quite closely.
    ///
    /// ```
    /// # use alexandria::{GLOBAL, Builder, Library, error::Result, utils::{Tag, TagSet, Path}, query::Query};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build().unwrap();
    /// # let tag1 = Tag::new("tag1", vec![1, 3, 1, 2]);
    /// # let tag2 = Tag::new("tag2", vec![13, 12]);
    /// let tags = TagSet::from(vec![tag1, tag2]);
    /// let iter = lib
    ///     .query_iter(GLOBAL, Query::tags().equals(tags))
    ///     .await?;
    /// iter.skip(5);
    /// let rec = iter.next().await;
    /// # Ok(()) }
    /// ```
    ///
    /// ## Garbage collection
    ///
    /// By default, garbage collection isn't locked for paths that are
    /// included in an iterator.  What this means is that any `delete`
    /// call can remove records that will at some point be accessed by
    /// the returned iterator, resulting in an `Err(_)` return.  To
    /// avoid this race condition, you can call `lock()` on the
    /// iterator, which blocks alexandria from cleaning the iternal
    /// record representation for items that are supposed to be
    /// accessed by the iterator.
    ///
    /// **Note:** `query` may still return "No such path" for these
    /// items, since they were already deleted from the tag cache.
    /// And a caveat worth mentioning: if the program aborts before
    /// the Iterator `drop` was able to run, the items will not be
    /// cleaned from disk and reloaded into cache on restart.
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn query_iter<S>(self: &Arc<Self>, id: S, q: Query) -> Result<QueryIterator>
    where
        S: Into<Session> + Debug,
    {
        let id = id.into();
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        Ok(QueryIterator::new(
            id,
            match q {
                Query::Path(ref p) => vec![p.clone()],
                Query::Tag(ref tq) => {
                    let tc = self.tag_cache.read().await;
                    match tq {
                        SetQuery::Intersect(ref tags) => tc.get_paths(id, |o| tags.intersect(o)),
                        SetQuery::Subset(ref tags) => {
                            // FIXME: I don't really know why this
                            // operation needs to be asymptotic, but
                            // some operations seem to be backwards?
                            // In either case, this works but we
                            // should figure out why this is.
                            tc.get_paths(id, |o| tags.subset(o) || o.subset(tags))
                        }
                        SetQuery::Equals(ref tags) => tc.get_paths(id, |o| tags.equality(o)),
                        SetQuery::Not(ref tags) => tc.get_paths(id, |o| tags.not(o)),
                    }
                }
                _ => unimplemented!(),
            },
            Arc::clone(self),
            q,
        ))
    }

    /// Subscribe to future database updates via a query filter
    ///
    /// When querying repeatedly isn't an option, or would lead to
    /// decreased performance, it's also possible to register a
    /// subscription.  They use the same mechanism as Queries to
    /// filter through tags and paths, but return a type that can be
    /// async-polled for updates.
    ///
    /// This doesn't give immediate access to the data, only the path
    /// that was changed, but can then be used to make a real query
    /// into the database to get an updated set of data.
    ///
    /// ```
    /// # use alexandria::{GLOBAL, Builder, Library, error::Result, utils::{Tag, TagSet, Path}, query::{Query, SetQuery}};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build().unwrap();
    /// # let my_tag = Tag::new("tag1", vec![1, 3, 1, 2]);
    /// let tags = TagSet::from(vec![my_tag]);
    /// let sub = lib.subscribe(GLOBAL, Query::tags().subset(tags)).await?;
    ///
    /// let path = sub.next().await;
    /// let new_data = lib.query(GLOBAL, Query::Path(path)).await?;
    /// # Ok(()) }
    /// ```
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn subscribe<S>(&self, id: S, q: Query) -> Result<Subscription>
    where
        S: Into<Session> + Debug,
    {
        let id = id.into();
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        Ok(self.subs.add_sub(q).await)
    }

    /// Check if a path exists for a particular session
    ///
    /// When inserting data, sometimes it's useful to check the path
    /// that was just inserted, or was inserted by a previous
    /// operation.  Because this is such a common operation, this
    /// utility function aims to make this workflow easier.
    ///
    /// If you want the actual type of a path node, use `query()` instead!
    pub async fn path_exists<S>(&self, id: S, p: Path) -> Result<bool>
    where
        S: Into<Session> + Debug,
    {
        self.query(id, Query::path(p)).await.map(|res| res.single())
    }
}
