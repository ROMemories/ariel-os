//! This module provides an event that can be waited for.
//!
use core::cell::Cell;

use crate::{threadlist::ThreadList, ThreadState};

/// A Event object.
///
/// An [`Event`] can be used to notify multiple threads that some event has happened.
///
/// An Event object manages an internal flag that can be set to true with the set() method and reset
/// to false with the clear() method. The wait() method blocks until the flag is set to true. The
/// flag is set to false initially.
pub struct Event {
    state: Cell<LockState>,
}

unsafe impl Sync for Event {}

enum LockState {
    Unlocked,
    Locked(ThreadList),
}

impl Event {
    /// Creates new **unset** Event.
    pub const fn new() -> Self {
        Self {
            state: Cell::new(LockState::Locked(ThreadList::new())),
        }
    }

    /// Creates new **set** Event.
    pub const fn new_set() -> Self {
        Self {
            state: Cell::new(LockState::Unlocked),
        }
    }

    /// Returns the current state.
    ///
    /// true if locked, false otherwise
    pub fn is_set(&self) -> bool {
        critical_section::with(|_| {
            let state = self.state.replace(LockState::Unlocked);
            let set = matches!(state, LockState::Unlocked);
            self.state.replace(state);
            set
        })
    }

    /// Wait for this [`Event`] to be set (blocking).
    ///
    /// If the event was set, this function returns directly.
    /// If the event was unset, this function will block the current thread until
    /// the event gets set elsewhere.
    ///
    /// # Panics
    ///
    /// Panics if this is called outside of a thread context.
    pub fn wait(&self) {
        critical_section::with(|cs| {
            let state = self.state.replace(LockState::Unlocked);
            match state {
                LockState::Unlocked => {
                    self.state.replace(state);
                }
                LockState::Locked(mut waiters) => {
                    waiters.put_current(cs, ThreadState::LockBlocked);
                    self.state.replace(LockState::Locked(waiters));
                }
            }
        });
    }

    /// Clear the event (non-blocking).
    ///
    /// If the event was set, it will be cleared and the function returns true.
    /// If the event was unset, the function returns false
    pub fn clear(&self) -> bool {
        critical_section::with(|_| {
            let state = self.state.replace(LockState::Unlocked);
            match state {
                LockState::Unlocked => {
                    self.state.replace(LockState::Locked(ThreadList::new()));
                    true
                }
                LockState::Locked(_) => {
                    self.state.replace(state);
                    false
                }
            }
        })
    }

    /// Set the event.
    ///
    /// If the event was unset, and there were waiters, all waiters will be
    /// woken up.
    /// If the event was already set, the function just returns.
    pub fn set(&self) {
        critical_section::with(|cs| {
            let state = self.state.replace(LockState::Unlocked);
            match state {
                LockState::Unlocked => {
                    self.state.replace(state);
                }
                LockState::Locked(mut waiters) => {
                    // unlock all waiters
                    while waiters.pop(cs).is_some() {}
                    self.state.replace(LockState::Unlocked);
                }
            }
        });
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}
