#![no_std]
#![feature(doc_auto_cfg)]

pub mod gpio;

#[doc(hidden)]
pub mod peripheral {
    pub use embassy_nrf::Peripheral;
}

#[cfg(feature = "external-interrupts")]
#[doc(hidden)]
pub mod extint_registry;

#[cfg(feature = "hwrng")]
#[doc(hidden)]
pub mod hwrng;

#[cfg(feature = "i2c")]
pub mod i2c;

#[doc(hidden)]
pub mod identity;

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "storage")]
#[doc(hidden)]
pub mod storage;

#[cfg(feature = "usb")]
#[doc(hidden)]
pub mod usb;

#[cfg(feature = "executor-interrupt")]
#[doc(hidden)]
pub use embassy_executor::InterruptExecutor as Executor;

#[cfg(feature = "executor-interrupt")]
#[cfg(context = "nrf52")]
riot_rs_embassy_common::executor_swi!(SWI0_EGU0);

#[cfg(feature = "executor-interrupt")]
#[cfg(context = "nrf5340")]
riot_rs_embassy_common::executor_swi!(EGU0);

use embassy_nrf::config::Config;

#[doc(hidden)]
pub use embassy_nrf::{interrupt, peripherals};

pub use embassy_nrf::OptionalPeripherals;

#[cfg(feature = "executor-interrupt")]
#[doc(hidden)]
pub static EXECUTOR: Executor = Executor::new();

#[doc(hidden)]
pub fn init() -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(Config::default());
    OptionalPeripherals::from(peripherals)
}
