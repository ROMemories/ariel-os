#![no_std]

use core::future::Future;

/// Represents a device providing sensor readings.
// FIXME: add a test to make sure this trait is object-safe
pub trait Sensor<R: Reading> {
    // FIXME: do we need a Send bound in the return type?
    fn read(&mut self) -> impl Future<Output = R>;
}

pub trait Reading {
    fn value(&self) -> PhysicalValue;

    fn values(&self) -> impl ExactSizeIterator<Item = PhysicalValue> {
        [self.value()].into_iter()
    }
}

// TODO: provide new() + getters instead of making fields public?
#[derive(Debug, Copy, Clone)]
pub struct PhysicalValue {
    pub value: i32,
    pub unit: PhysicalUnit,
    pub scale: i8,
}

// Built upon https://doc.riot-os.org/phydat_8h_source.html
// and https://bthome.io/format/#sensor-data
// and https://www.rfc-editor.org/rfc/rfc8798.html
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum PhysicalUnit {
    Celsius,
    // TODO: add other units
}
