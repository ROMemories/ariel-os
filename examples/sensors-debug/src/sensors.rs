//! This whole module should eventually be codegened.

use riot_rs::arch::peripherals;

/// Type alias of this sensor instance
#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
pub type PushButton_BUTTON_1 = riot_rs_builtin_sensors::push_buttons::PushButton;

// Instantiate the sensor driver
#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
pub static BUTTON_1: PushButton_BUTTON_1 = PushButton_BUTTON_1::new(Some("btn1"));

// Store a static reference in the sensor distributed slice
#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
#[riot_rs::reexports::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::reexports::linkme)]
static BUTTON_1_REF: &'static dyn riot_rs::sensors::Sensor = &BUTTON_1;

// Set the sensor initialization to run at startup
#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
#[riot_rs::spawner(autostart, peripherals)]
fn BUTTON_1_init(spawner: riot_rs::Spawner, peripherals: crate::pins::BUTTON_1Peripherals) {
    let mut config = riot_rs_builtin_sensors::push_buttons::Config::default();

    let pull = riot_rs::gpio::Pull::Up;
    let input = riot_rs::gpio::Input::new(peripherals.p, pull);

    BUTTON_1.init(spawner, input, config);
}

#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
#[riot_rs::task(autostart)]
async fn BUTTON_1_run() {
    BUTTON_1.run().await
}
