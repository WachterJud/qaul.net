//! Atomic pointer abstraction
//!
//! This interface is far too useful for qaul.net as a whole to stay
//! in this file, so we should pull it out into it's own crate at some
//! point.  But for now...

use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};
use std::{cmp::PartialEq, ops::Deref};

/// An alias for a referenced pointer
pub(crate) struct Ref<T> {
    inner: Box<Arc<T>>,
}

impl<T> Deref for Ref<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A safe atomic pointer wrapper
#[derive(Clone, Debug)]
pub(crate) struct AtomPtr<T> {
    inner: Arc<AtomicPtr<Arc<T>>>,
}

// Implement Default for all T that implement default
impl<T: Default> Default for AtomPtr<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> PartialEq for AtomPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.get_ref().inner, &other.get_ref().inner)
    }
}

impl<T> AtomPtr<T> {
    /// Create a new atomic pointer for a type
    pub(crate) fn new(t: T) -> Self {
        let arc = Arc::new(t);
        let ptr = Box::into_raw(Box::new(arc));
        let inner = Arc::new(AtomicPtr::from(ptr));
        Self { inner }
    }

    /// Get an immutable reference to the current value
    pub(crate) fn get_ref(&self) -> Ref<T> {
        let ptr = self.inner.load(Ordering::Relaxed);
        let b = unsafe { Box::from_raw(ptr) };

        let arc = Arc::clone(&*b);
        std::mem::forget(b);

        Ref {
            inner: Box::new(arc),
        }
    }

    /// Swap the data entry with a new value, returning the old
    pub(crate) fn swap(&self, new: T) -> Ref<T> {
        let ptr = self.inner.load(Ordering::Relaxed);
        self.inner.swap(ptr, Ordering::Relaxed);

        let b = unsafe { Box::from_raw(ptr) };
        let arc = Arc::clone(&*b);
        std::mem::forget(b);

        Ref {
            inner: Box::new(arc),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct TestStruct {
    name: String,
}

#[test]
fn cloned() {
    let ts = TestStruct {
        name: "Hello".into(),
    };

    let ptr1 = AtomPtr::new(ts);
    let ptr2 = ptr1.clone();

    assert_eq!(ptr1, ptr2);
}

#[test]
fn swap() {
    let ts1 = TestStruct {
        name: "Hello 1".into(),
    };

    let ts2 = TestStruct {
        name: "Hello 2".into(),
    };

    let ptr = AtomPtr::new(ts1.clone());
    let still_ts1 = ptr.swap(ts2);

    assert_eq!(ts1, *still_ts1.as_ref());
}
