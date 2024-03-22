#![no_std]
#![feature(error_in_core)]

use core::future::Future;

/// Represents a device providing sensor readings.
// FIXME: add a test to make sure this trait is object-safe
pub trait Sensor<R: Reading> {
    // FIXME: do we need a Send bound in the return type (or rely on consumers using SendCell when
    // needed)?
    fn read(&mut self) -> impl Future<Output = Result<R, ReadingError>>;

    // TODO: can we make this a trait const instead? way cause object safety issues
    fn value_scale() -> i8;

    // TODO: can we make this a trait const instead? way cause object safety issues
    fn unit() -> PhysicalUnit;
}

pub trait Reading {
    fn value(&self) -> PhysicalValue;

    fn values(&self) -> impl ExactSizeIterator<Item = PhysicalValue> {
        [self.value()].into_iter()
    }
}

// FIXME: make this a proper error type
#[derive(Debug)]
pub enum ReadingError {}

impl core::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "error when accessing a sensor reading")
    }
}

impl core::error::Error for ReadingError {}

pub type ReadingResult<R> = Result<R, ReadingError>;

// TODO: provide new() + getters instead of making fields public?
#[derive(Debug, Copy, Clone)]
pub struct PhysicalValue {
    pub value: i32,
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
