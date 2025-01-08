//! Pass non-Send objects around on same executor.
//!
//! This module provides [`SendCell`], a structure that allows passing around
//! non-Send objects from one async task to another, if they are on the same
//! executor. This is allowed because embassy executors are single threaded.
//!
//! [`SendCell`] checks for the correct executor *at runtime*.

// SAFETY:
// SendCell guarantees at runtime that its content stays on the same embassy
// executor. Those are single threaded, so it is guaranteed that the content
// stays on the same thread.
// While `SendCell::get()` allows passing any `Spawner` object, those are `!Send`,
// thus they are guaranteed to be for the current Executor.
unsafe impl<T> Send for SendCell<T> {}

/// A cell that allows sending of non-Send types *if they stay on the same executor*.
///
/// This is *checked at runtime*.
///
/// Both [`new()`](SendCell::new) and [`get()`](SendCell::get) have async versions ([`new_async()`](SendCell::new_async) and [`get_async()`](SendCell::get_async)) that get a
/// handle for the current [`Spawner`] themselves. They internally call the non-async versions. Use
/// the sync versions if a [`Spawner`] object is available or the async versions cannot be used,
/// e.g., in closures. Otherwise, the async versions are also fine.
#[derive(Debug)]
pub struct SendCell<T> {
    #[cfg(feature = "threading")]
    tid: ariel_os_threads::ThreadId,
    inner: T,
}

impl<T> SendCell<T> {
    /// Creates a new [`SendCell`].
    pub fn new(inner: T) -> Self {
        Self {
            #[cfg(feature = "threading")]
            tid: ariel_os_threads::current_pid().unwrap(),
            inner,
        }
    }

    /// Gets the contents of this [`SendCell`].
    pub fn get(&self) -> Option<&T> {
        #[cfg(not(feature = "threading"))]
        return Some(&self.inner);

        #[cfg(feature = "threading")]
        if ariel_os_threads::current_pid() == Some(self.tid) {
            Some(&self.inner)
        } else {
            None
        }
    }
}
