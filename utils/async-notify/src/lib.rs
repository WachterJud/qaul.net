//! A utility wrapper type for notifying async tasks about mutation of data they're interested in.

#![doc(html_favicon_url = "https://qaul.net/favicon.ico")]
#![doc(html_logo_url = "https://qaul.net/img/qaul_icon-128.png")]

use std::ops::{Deref, DerefMut};
use std::task::Waker;

/// A wrapper which wakes tasks on mutable accesses to the wrapped value.
///
/// This can be used to transparently notify an asynchronous task that it
/// should, for example, check for more work in a queue or try again to
/// acquire a lock.
#[derive(Default, Debug, Clone)]
pub struct Notify<T> {
    inner: T,
    waker: Option<Waker>,
}

impl<T> Deref for Notify<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Notify<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.waker.as_ref().map(|w| w.wake_by_ref());
        &mut self.inner
    }
}

impl<T> Notify<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, waker: None }
    }

    /// Check whether or not this `Notify` has a registered `Waker`.
    ///
    /// This function is implemented as an associated function rather than a
    /// method to avoid conflicts with methods on the wrapped type.
    /// Call it as `Notify::has_waker()`.
    pub fn has_waker(ptr: &Notify<T>) -> bool {
        ptr.waker.is_some()
    }

    /// Get a copy of the registered `Waker` for this `Notify`.
    ///  
    /// This function is implemented as an associated function rather than a
    /// method to avoid conflicts with methods on the wrapped type.
    /// Call it as `Notify::waker()`.
    pub fn waker(ptr: &mut Notify<T>) -> Option<Waker> {
        ptr.waker.as_ref().map(|w| w.clone())
    }

    /// Call wake on the waker, if it's a waker, yehaa!
    #[inline]
    pub fn wake(ptr: &mut Notify<T>) {
        if let Some(ref w) = ptr.waker {
            w.clone().wake();
        }
    }

    /// Register a `Waker` to be woken upon mutable accesses to the wrapped value.
    ///  
    /// This function is implemented as an associated function rather than a
    /// method to avoid conflicts with methods on the wrapped type.
    /// Call it as `Notify::register_waker()`.
    ///
    /// # Panics
    /// Panics if there is an already registered `Waker`.
    /// Use `Notify::has_waker` to check the state before using this.
    #[inline]
    pub fn register_waker(ptr: &mut Notify<T>, waker: &Waker) {
        if !Notify::has_waker(ptr) {
            ptr.waker = Some(waker.clone())
        }
    }

    /// Removes and returns the `Waker` registered to this `Notify`.
    ///  
    /// This function is implemented as an associated function rather than a
    /// method to avoid conflicts with methods on the wrapped type.
    /// Call it as `Notify::clear_waker()`.
    pub fn clear_waker(ptr: &mut Notify<T>) -> Option<Waker> {
        ptr.waker.take()
    }

    /// Consumes the `Notify`, dropping any associated `Waker` and
    /// returning the inner value without notifying the `Waker`.
    ///  
    /// This function is implemented as an associated function rather than a
    /// method to avoid conflicts with methods on the wrapped type.
    /// Call it as `Notify::into_inner()`.
    pub fn into_inner(ptr: Notify<T>) -> T {
        ptr.inner
    }
}
