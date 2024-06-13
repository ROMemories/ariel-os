//! Provides a [`Sensor`] trait abstracting over implementation details of a sensor.
//!
//! To obtain a measurement and make sense of the result, two methods are required:
//! [`Sensor::measure()`] and [`Sensor::reading_infos()`].
//!
//! - [`Sensor::measure()`] returns a [`PhysicalValues`], a data "tuple" containing values returned
//! by the sensor device.
//! - The [`ReadingInfos`] type, returned by [`Sensor::reading_infos()`], tells what physical value
//! each value from that tuple corresponds to, using [`Label`].
//! For instance, this allows to disambiguate the values provided by a temperature & humidity
//! sensor.
//! The [`ReadingInfos`] are fixed for a given sensor driver, allowing consumers to display
//! information about the physical values a sensor measures without triggering a measurement.
//!
//! To avoid float handling, values returned by [`Sensor::measure()`] are integers, and a
//! fixed scaling value is provided in [`ReadingInfo`], for each [`PhysicalValue`] returned.
//! See [`PhysicalValue`] for more details.

use core::{any::Any, future::Future};

// TODO: use a zero-copy channel instead?
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};

use crate::{Category, Label, PhysicalUnit};

pub use crate::{
    physical_value::{AccuracyError, PhysicalValue},
    Reading,
};

/// Represents a device providing sensor readings.
// TODO: introduce a trait currently deferring to Any
pub trait Sensor: Any + Send + Sync {
    // TODO: add a link to an explanation of the setup file
    /// Triggers a measurement, waits for the result and returns it asynchronously.
    /// Depending on the sensor and the driver configuration, this may use a sensor interrupt or
    /// data polling.
    ///
    /// Interpretation of the values returned requires data from [`Sensor::reading_infos()`] as
    /// well.
    /// See [the module level documentation](crate::sensor) for more.
    ///
    /// # Note
    ///
    /// As this method is non-dispatchable, the
    /// [`riot_rs::sensors::measure!()`](riot_rs_macros::measure_sensor!) macro must be used
    /// instead of calling this method directly, this macro requires a  properly configured setup
    /// file.
    fn measure(&self) -> impl Future<Output = ReadingResult<impl Reading>>
    where
        Self: Sized;

    // FIXME: rename this
    /// Provides information about the values returned by [`Sensor::measure()`].
    #[must_use]
    fn reading_infos(&self) -> ReadingInfos;

    // TODO: allow to sleep? set_mode() a u8 atomic
    /// Enables or disables the sensor driver.
    fn set_enabled(&self, enabled: bool);

    /// Returns whether the sensor driver is enabled.
    #[must_use]
    fn enabled(&self) -> bool;

    /// Returns the categories the sensor is part of.
    #[must_use]
    fn categories(&self) -> &'static [Category];

    /// String label of the sensor instance.
    ///
    /// For instance, in the case of a temperature sensor, this allows to specify whether it is
    /// placed indoor or outdoor.
    #[must_use]
    fn label(&self) -> Option<&'static str>;

    /// Returns a human-readable name of the sensor.
    // TODO: i18n?
    #[must_use]
    fn display_name(&self) -> Option<&'static str>;

    /// Returns the hardware sensor part number.
    #[must_use]
    fn part_number(&self) -> &'static str;

    /// Returns the sensor driver version number.
    #[must_use]
    fn version(&self) -> u8;
}

impl dyn Sensor {
    pub fn downcast_ref<T: Sensor>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

riot_rs_macros::define_count_adjusted_enums!();

/// Provides information about a [`PhysicalValue`].
#[derive(Debug, Copy, Clone, serde::Serialize)]
pub struct ReadingInfo {
    label: Label,
    scaling: i8,
    unit: PhysicalUnit,
}

impl ReadingInfo {
    #[must_use]
    pub fn new(label: Label, scaling: i8, unit: PhysicalUnit) -> Self {
        Self {
            label,
            scaling,
            unit,
        }
    }

    #[must_use]
    pub fn label(&self) -> Label {
        self.label
    }

    #[must_use]
    pub fn scaling(&self) -> i8 {
        self.scaling
    }

    #[must_use]
    pub fn unit(&self) -> PhysicalUnit {
        self.unit
    }
}

/// Represents errors happening when accessing a sensor reading.
// TODO: is it more useful to indicate the error nature or whether it is temporary or permanent?
#[derive(Debug)]
pub enum ReadingError {
    /// The sensor is disabled.
    Disabled,
    /// Cannot access the sensor (e.g., because of a bus error).
    SensorAccess,
}

impl core::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // FIXME: update this
        write!(f, "error when accessing a sensor reading")
    }
}

impl core::error::Error for ReadingError {}

pub type ReadingResult<R> = Result<R, ReadingError>;

/// Returns the result of calling [`Sensor::measure()`] on the sensor concrete type.
///
/// Downcasts the provided sensor (which must be implementing the [`Sensor`] trait) to its concrete
/// type, and calls the async, non-dispatchable `Sensor::measure()` method on it.
///
/// This macro needs to be provided with the sensor and with the list of existing sensor concrete
/// types (at least one).
///
/// # Panics
///
/// Panics if the concrete type of the sensor was not present in the list of types provided.
// Should not be used by users directly, users should use the `riot_rs::read_sensor!()` proc-macro
// instead.
#[doc(hidden)]
#[macro_export]
macro_rules! _measure_sensor {
    (
        $sensor:path,
        $(
            $(#[$sensor_attr:meta])*
            $sensor_type:path
        ),*
        $(,)?
    ) => {
        {
            use $crate::{sensor::ReadingResult, Sensor};

            // As `Sensor::measure()` is non-dispatchable, we have to downcast
            async fn __measure_sensor(sensor: &dyn Sensor) -> ReadingResult<impl Reading> {
                $(
                $(#[$sensor_attr])*
                if let Some(sensor) = sensor.downcast_ref::<$sensor_type>() {
                    return sensor.measure().await;
                }
                )*

                // Every possible sensor concrete types must have been provided.
                unreachable!();
            }

            __measure_sensor($sensor)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assert that the Sensor trait is object-safe
    static _SENSOR_REFS: &[&dyn Sensor] = &[];
}
