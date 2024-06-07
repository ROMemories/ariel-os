use core::future::Future;

use embassy_sync::channel::Receiver;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};
use portable_atomic::{AtomicBool, AtomicI32, Ordering};

use crate::{
    sensor::{PhysicalValue, ReadingResult},
    Reading, Sensor,
};

pub struct Watcher<'sensor> {
    sensor: &'sensor dyn Sensor,
    channel: Channel<CriticalSectionRawMutex, Notification, 1>,
    lower_threshold: AtomicI32,
    lower_threshold_enabled: AtomicBool, // TODO: use an atomic bitset
}

impl<'sensor> Watcher<'sensor> {
    #[must_use]
    pub const fn new(sensor: &'sensor dyn Sensor) -> Self {
        Self {
            sensor,
            channel: Channel::new(),
            lower_threshold: AtomicI32::new(0),
            lower_threshold_enabled: AtomicBool::new(false),
        }
    }

    pub fn sensor(&self) -> &dyn Sensor {
        self.sensor
    }

    // TODO: support some hysteresis
    pub fn set_threshold(&self, kind: ThresholdKind, value: PhysicalValue) {
        match kind {
            ThresholdKind::Lower => self.lower_threshold.store(value.value(), Ordering::Release),
            _ => {
                // TODO: should we return an error instead?
            }
        }
    }

    // TODO: merge this with set_threshold?
    pub fn set_threshold_enabled(&self, kind: ThresholdKind, enabled: bool) {
        match kind {
            ThresholdKind::Lower => self
                .lower_threshold_enabled
                .store(enabled, Ordering::Release),
            _ => {
                // TODO: should we return an error instead?
            }
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> NotificationReceiver {
        // TODO: receiver competes for notification: limit the number of receivers to 1?
        self.channel.receiver()
    }
}

#[macro_export]
macro_rules! new_watcher {
    ($sensor:ident) => {
        // #[embassy_executor::task]
        // async fn watcher_task(watcher: &'static Watcher<'_>, value_index: usize) {
        //     loop {
        //         if watcher.lower_threshold_enabled.load(Ordering::Acquire) {
        //             if let Ok(value) = watcher.sensor_read().await {
        //                 if value.values().nth(value_index).unwrap().value()
        //                     > watcher.lower_threshold.load(Ordering::Acquire)
        //                 {
        //                     // FIXME: should this be Lower or Higher?
        //                     let _ = watcher
        //                         .channel
        //                         .try_send(Notification::Threshold(ThresholdKind::Lower));
        //                     riot_rs_debug::println!("Value > lower threshold: {:?}", value);
        //                 }
        //             }
        //         }
        //         // TODO: make this duration configurable?
        //         // Avoid busy looping and allow other users to lock the mutex
        //         Timer::after(Duration::from_millis(100)).await;
        //     }
        // }

        {
            $crate::watcher::Watcher::new($sensor)
        }
    }
}

// spawner.spawn(watcher(&self)).unwrap();

/// A notification provided by a sensor driver.
// TODO: should we pass the value as well? that may be difficult because of the required generics
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum Notification {
    ReadingAvailable,
    Threshold(ThresholdKind),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum ThresholdKind {
    Lower,
    Higher,
}

// TODO: tune the channel size
pub type NotificationReceiver<'a> = Receiver<'a, CriticalSectionRawMutex, Notification, 1>;
