//! Dummy module used to satisfy platform-independent tooling.
// TODO: redirect to the manufacturer-specific crate documentation when we publish it, and
// mark every item in this dummy module `doc(hidden)`

#![allow(
    clippy::missing_errors_doc,
    reason = "this module is hidden in the docs"
)]
#![expect(clippy::duplicated_attributes, reason = "Clippy bug #13355")]
#![allow(
    clippy::module_name_repetitions,
    reason = "this dummy module mimics an other, actual module"
)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "this dummy module mimics an other, actual module"
)]

mod executor;
pub mod gpio;

pub mod peripheral {
    pub use embassy_hal_internal::Peripheral;
}

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "i2c")]
pub mod i2c;

pub mod identity {
    use riot_rs_embassy_common::identity;

    pub type DeviceId = identity::NoDeviceId<identity::NotImplemented>;
}

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "usb")]
pub mod usb;

pub use executor::{Executor, Spawner};

/// Dummy type.
///
/// See the `OptionalPeripherals` type of your Embassy architecture crate instead.
pub struct OptionalPeripherals;

/// Dummy type.
pub struct Peripherals;

impl From<Peripherals> for OptionalPeripherals {
    fn from(_peripherals: Peripherals) -> Self {
        Self {}
    }
}

#[must_use]
pub fn init() -> OptionalPeripherals {
    unimplemented!();
}

pub struct SWI;
