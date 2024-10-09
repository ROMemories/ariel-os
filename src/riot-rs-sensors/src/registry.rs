//! Provides a sensor driver instance registry, allowing to register sensor driver instances and
//! access them in a centralized location.

use core::future::Future;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Sender},
    signal::Signal,
};

use crate::{
    sensor::{PhysicalValues, ReadingResult},
    Sensor,
};

/// Stores references to registered sensor driver instances.
///
/// To register a sensor driver instance, insert a `&'static` into this [distributed
/// slice](linkme).
/// The sensor driver will therefore need to be statically allocated, to be able to obtain a
/// `&'static`.
// Exclude this from the users' documentation, to force users to use `Registry::sensors()` instead,
// for easier forward compatibility with possibly non-static references.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static SENSOR_REFS: [&'static dyn Sensor] = [..];

/// The global registry instance.
pub static REGISTRY: Registry = Registry::new();

#[doc(hidden)]
pub static TRIGGER_MEASUREMENT: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();
#[doc(hidden)]
pub static RECEIVE_READING: Channel<CriticalSectionRawMutex, ReadingResult<PhysicalValues>, 3> =
    Channel::new();

/// The sensor driver instance registry.
pub struct Registry {}

impl Registry {
    // The constructor is private to make the registry a singleton.
    const fn new() -> Self {
        Self {}
    }

    /// Returns an iterator over registered sensor driver instances.
    pub fn sensors(&self) -> impl Iterator<Item = &'static dyn Sensor> {
        // Returning an iterator instead of the distributed slice directly would allow us to chain
        // another source of sensor driver instances in the future, if we decided to support
        // dynamically-allocated sensor driver instances.
        SENSOR_REFS.iter().copied()
    }

    pub async fn measure(&self, sensor: &'static dyn Sensor) -> ReadingResult<PhysicalValues> {
        // FIXME: use a oneshort channel with an owned sender instead
        let response_sender = RECEIVE_READING.sender();

        let _ = TRIGGER_MEASUREMENT.try_send(Request {
            sensor,
            response_sender,
        });

        // FIXME: this is definitely not guaranteed to return the reading for the measurement
        // requested above.
        RECEIVE_READING.receive().await
    }
}

pub struct Request<'ch> {
    pub sensor: &'static dyn Sensor,
    pub response_sender: Sender<'ch, CriticalSectionRawMutex, ReadingResult<PhysicalValues>, 3>,
}
