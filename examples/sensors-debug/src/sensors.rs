//! This whole module should eventually be codegened.

use riot_rs::arch::peripherals;

pub use sensors::*;

/// Type alias of this sensor instance
#[cfg(feature = "button-readings")]
pub type PushButton_BUTTON_1 = riot_rs_builtin_sensors::push_buttons::PushButton;

// Instantiate the sensor driver
#[cfg(feature = "button-readings")]
pub static BUTTON_1: PushButton_BUTTON_1 = PushButton_BUTTON_1::new(Some("btn1"));

// Store a static reference in the sensor distributed slice
#[cfg(feature = "button-readings")]
#[riot_rs::reexports::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::reexports::linkme)]
static BUTTON_1_REF: &'static dyn riot_rs::sensors::Sensor = &BUTTON_1;

#[cfg(feature = "button-readings")]
#[riot_rs::task(autostart)]
async fn BUTTON_1_run() {
    BUTTON_1.run().await
}

#[cfg(context = "nrf52840dk")]
mod sensors {
    use super::BUTTON_1;

    // Set the sensor initialization to run at startup
    #[cfg(feature = "button-readings")]
    #[riot_rs::spawner(autostart, peripherals)]
    fn BUTTON_1_init(spawner: riot_rs::Spawner, peripherals: crate::pins::BUTTON_1Peripherals) {
        let mut config = riot_rs_builtin_sensors::push_buttons::Config::default();

        let pull = riot_rs::gpio::Pull::Up;
        let input = riot_rs::gpio::Input::new(peripherals.p, pull);

        BUTTON_1.init(spawner, input, config);
    }
}

#[cfg(context = "st-nucleo-h755zi-q")]
mod sensors {
    use super::BUTTON_1;

    // Set the sensor initialization to run at startup
    #[cfg(feature = "button-readings")]
    #[riot_rs::spawner(autostart, peripherals)]
    fn BUTTON_1_init(spawner: riot_rs::Spawner, peripherals: crate::pins::BUTTON_1Peripherals) {
        let mut config = riot_rs_builtin_sensors::push_buttons::Config::default();
        config.active_low = false;

        let pull = riot_rs::gpio::Pull::Down;
        let input = riot_rs::gpio::Input::new(peripherals.p, pull);

        BUTTON_1.init(spawner, input, config);
    }
}
