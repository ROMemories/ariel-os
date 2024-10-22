use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};

use crate::sensor::{ReadingError, ReadingResult, ReadingWaiter, Values};

// TODO: move this
// TODO: rename this
/// Intended for sensor driver implementors only.
pub struct SensorSignaling {
    trigger: Signal<CriticalSectionRawMutex, ()>,
    reading_channel: Channel<CriticalSectionRawMutex, ReadingResult<Values>, 1>,
}

impl SensorSignaling {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            trigger: Signal::new(),
            reading_channel: Channel::new(),
        }
    }

    pub fn trigger_measurement(&self) {
        // Remove the possibly lingering reading.
        self.reading_channel.clear();

        self.trigger.signal(());
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
