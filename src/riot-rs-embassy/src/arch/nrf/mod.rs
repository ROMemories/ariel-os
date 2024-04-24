pub mod gpio;

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "internal-temp")]
pub mod internal_temp;

#[cfg(feature = "usb")]
pub mod usb;

pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(context = "nrf52")]
pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;

#[cfg(context = "nrf5340")]
pub use embassy_nrf::interrupt::EGU0 as SWI;

pub use embassy_nrf::{config::Config, interrupt, peripherals, OptionalPeripherals};

#[cfg(context = "nrf52")]
#[interrupt]
unsafe fn SWI0_EGU0() {
    // SAFETY:
    // - called from ISR
    // - not called before `start()`, as the interrupt is enabled by `start()`
    //   itself
    unsafe { crate::EXECUTOR.on_interrupt() }
}

#[cfg(context = "nrf5340")]
#[interrupt]
unsafe fn EGU0() {
    unsafe { crate::EXECUTOR.on_interrupt() }
}

// FIXME: move this, and use the arch re-export of the gpio module
pub type PushButtonNrf =
    riot_rs_sensors::push_buttons::PushButton<embassy_nrf::gpio::Input<'static>>;

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(config);
    OptionalPeripherals::from(peripherals)
}
