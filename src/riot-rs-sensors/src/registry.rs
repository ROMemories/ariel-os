//! Provides a sensor registry, allowing to register sensors and access them in a centralized
//! location.

use crate::Sensor;

/// Stores references to registered sensors.
///
/// To register a sensor, insert it to this [distributed slice](linkme).
/// The sensor will need to be statically allocated, to be able to obtain a `'static` reference to
/// it.
// Exclude this from the users' documentation, to force users to use `Registry::sensors()` instead.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static SENSOR_REFS: [&'static dyn Sensor] = [..];

/// The sensor registry instance.
pub static REGISTRY: Registry = Registry::new();

/// The sensor registry.
pub struct Registry {}

impl Registry {
    // The constructor is private to make the registry a singleton.
    const fn new() -> Self {
        Self {}
    }

    /// Returns an iterator over registered sensors.
    pub fn sensors(&self) -> impl Iterator<Item = &'static dyn Sensor> {
        // Returning an iterator instead of the distributed slice directly would allow us to chain
        // another source of sensors in the future, if we decided to support dynamically-allocated
        // sensors
        SENSOR_REFS.iter().copied()
    }
}
