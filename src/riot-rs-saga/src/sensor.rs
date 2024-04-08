use core::{any::Any, future::Future};

/// Represents a device providing sensor readings.
// FIXME: add a test to make sure this trait is object-safe
pub trait Sensor: Any + Send + Sync {
    // FIXME: do we need a Send bound in the return type (or rely on consumers using SendCell when
    // needed)?
    async fn read(&self) -> ReadingResult<PhysicalValue>
    where
        Self: Sized;
    // fn read<R>(&mut self) -> R where R: Future<Output = Result<PhysicalValue, ReadingError>>, Self: Sized;
    // fn read(&mut self) -> ReadFut;

    // TODO: can we make this a trait const instead? may cause object safety issues
    fn value_scale() -> i8
    where
        Self: Sized;

    // TODO: can we make this a trait const instead? may cause object safety issues
    fn unit() -> PhysicalUnit
    where
        Self: Sized;

    // TODO: i18n?
    fn display_name() -> Option<&'static str>
    where
        Self: Sized;

    fn part_number() -> &'static str
    where
        Self: Sized;

    fn version() -> u8
    where
        Self: Sized;
}

pub struct ReadFut {}

pub trait Reading {
    fn value(&self) -> PhysicalValue;

    fn values(&self) -> impl ExactSizeIterator<Item = PhysicalValue> {
        [self.value()].into_iter()
    }
}

// TODO: is it more useful to indicate the error nature or whether it is temporary or permanent?
#[derive(Debug)]
pub enum ReadingError {}

impl core::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "error when accessing a sensor reading")
    }
}

impl core::error::Error for ReadingError {}

pub type ReadingResult<R> = Result<R, ReadingError>;

// TODO: add a timestamp?
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
