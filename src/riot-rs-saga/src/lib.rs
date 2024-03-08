#![no_std]

/// Represents a device providing sensor readings.
///
/// The generic `N` is the count of `PhysicalValue`s a single reading is comprised of; this would
/// be `1` for a temperature sensor and `3` for a 3-axis accelerometer.
// FIXME: add a test to make sure this trait is object-safe
pub trait Sensor<const N: usize> {
    // FIXME: what to use this for
    fn initialize();

    // FIXME: do we need a Send bound in the return type?
    fn read(&mut self) -> impl core::future::Future<Output = Reading<N>>;
}

#[derive(Debug)]
pub struct Reading<const N: usize>(pub [PhysicalValue; N]);

// TODO: provide new() + getters instead of making fields public?
#[derive(Debug)]
pub struct PhysicalValue {
    pub value: i16, // FIXME
    pub unit: PhysicalUnit,
    pub scale: i8,
}

// Built upon https://doc.riot-os.org/phydat_8h_source.html
// and https://bthome.io/format/#sensor-data
#[derive(Debug)]
#[non_exhaustive]
pub enum PhysicalUnit {
    Celsius,
    // TODO: add other units
}
