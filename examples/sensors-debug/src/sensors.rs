//! This whole module should eventually be codegened.

use riot_rs::arch::peripherals;

pub use sensors::*;

/// Type alias of this sensor instance
#[cfg(feature = "button-readings")]
pub type PushButton_BUTTON_1 = riot_rs_builtin_sensors::push_button::PushButton;

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

/// Type alias of this sensor instance
pub type Lsm303agr_ACCEL = riot_rs_builtin_sensors::lsm303agr::Lsm303agrI2c;

// Instantiate the sensor driver
pub static ACCEL: Lsm303agr_ACCEL = Lsm303agr_ACCEL::new(Some("accel"));

// Store a static reference in the sensor distributed slice
#[riot_rs::reexports::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::reexports::linkme)]
static ACCEL_REF: &'static dyn riot_rs::sensors::Sensor = &ACCEL;

#[cfg(context = "nrf52840dk")]
mod sensors {
    use super::BUTTON_1;

    // Set the sensor initialization to run at startup
    #[cfg(feature = "button-readings")]
    #[riot_rs::spawner(autostart, peripherals)]
    fn BUTTON_1_init(spawner: riot_rs::Spawner, peripherals: crate::pins::BUTTON_1Peripherals) {
        let mut config = riot_rs_builtin_sensors::push_button::Config::default();

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
        let mut config = riot_rs_builtin_sensors::push_button::Config::default();
        config.active_low = false;

        let pull = riot_rs::gpio::Pull::Down;
        let input = riot_rs::gpio::Input::new(peripherals.p, pull);

        BUTTON_1.init(spawner, input, config);
    }
}

#[cfg(context = "microbit-v2")]
mod sensors {
    use super::ACCEL;

    #[riot_rs::task(autostart, peripherals)]
    async fn ACCEL_run(peripherals: riot_rs_builtin_sensors::lsm303agr::Peripherals) {
        let mut config = riot_rs_builtin_sensors::lsm303agr::Config::default();

        let mut i2c_device =
            riot_rs::i2c::controller::I2cDevice::new(crate::buses::I2C0.get().unwrap());

        let spawner = riot_rs::Spawner::for_current_executor().await;
        ACCEL.init(spawner, peripherals, i2c_device, config).await;

        ACCEL.run().await
    }
}
