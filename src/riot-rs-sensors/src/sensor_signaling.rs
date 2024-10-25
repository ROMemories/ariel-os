use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use embassy_time::{Duration, Timer};
use portable_atomic::{AtomicBool, Ordering};

use crate::sensor::{ReadingError, ReadingResult, ReadingWaiter, Values};

// TODO: rename this
/// Intended for sensor driver implementors only.
pub struct SensorSignaling {
    trigger: Waiter,
    reading_channel: Channel<CriticalSectionRawMutex, ReadingResult<Values>, 1>,
}

impl SensorSignaling {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            trigger: Waiter::new(),
            reading_channel: Channel::new(),
        }
    }

    pub fn trigger_measurement(&self) {
        // Remove the possibly lingering reading.
        self.reading_channel.clear();

        self.trigger.signal();
    }

    pub async fn wait_for_trigger(&self) {
        self.trigger.wait().await;
    }

    pub async fn signal_reading(&self, reading: Values) {
        self.reading_channel.send(Ok(reading)).await;
    }

    pub async fn signal_reading_err(&self, reading_err: ReadingError) {
        self.reading_channel.send(Err(reading_err)).await;
    }

    pub fn wait_for_reading(&'static self) -> ReadingWaiter {
        ReadingWaiter::Waiter {
            waiter: self.reading_channel.receive(),
        }
    }
}

#[derive(Debug)]
struct Waiter {
    signaled: AtomicBool,
}

impl Waiter {
    pub const fn new() -> Self {
        Self {
            signaled: AtomicBool::new(false),
        }
    }

    pub fn signal(&self) {
        self.signaled.store(true, Ordering::Release);
    }

    // FIXME: return a more efficient Future, that does not rely on Timer, but is also small
    // memory-wise.
    pub async fn wait(&self) {
        // Wait for the Waiter to be signaled, and reset it when that happens.
        while self
            .signaled
            .compare_exchange_weak(true, false, Ordering::Release, Ordering::Acquire)
            .is_err()
        {
            Timer::after(Duration::from_millis(1)).await;
        }
    }

    pub fn reset(&self) {
        self.signaled.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_type_sizes() {
        let values_size = size_of::<Values>();

        assert_eq!(values_size, 6 * size_of::<u32>());
        assert_eq!(size_of::<Waiter>(), size_of::<u8>());
        assert_eq!(align_of::<Waiter>(), 1);
        assert_eq!(
            size_of::<Channel<CriticalSectionRawMutex, ReadingResult<Values>, 1>>(),
            values_size + 16 * size_of::<u32>()
        );
        assert_eq!(size_of::<SensorSignaling>(), values_size + 18 * size_of::<u32>());
    }
}
