//! This whole module should eventually be codegened.

use riot_rs::arch;

// Using the OnceCell from once_cell instead of the one from core because it supports
// critical sections.
// TODO: move this to a `bus` module?
pub static I2C0: once_cell::sync::OnceCell<
    embassy_sync::mutex::Mutex<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        arch::i2c::controller::I2c,
    >,
> = once_cell::sync::OnceCell::new();

#[riot_rs::reexports::linkme::distributed_slice(riot_rs::INIT_TASKS)]
#[linkme(crate = riot_rs::reexports::linkme)]
pub fn init(mut peripherals: &mut arch::OptionalPeripherals) {
    let peripherals: crate::pins::BusPeripherals = {
        use riot_rs::define_peripherals::TakePeripherals;
        peripherals.take_peripherals()
    };

    let mut config = arch::i2c::controller::Config::default();
    config.frequency = arch::i2c::controller::Frequency::_100k;

    let i2c =
        arch::i2c::controller::TWISPI0::new(peripherals.i2c0_sda, peripherals.i2c0_scl, config);

    let _ = I2C0.set(embassy_sync::mutex::Mutex::new(i2c));
}
