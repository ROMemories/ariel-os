use riot_rs::arch::{i2c, peripherals};

#[cfg(any(context = "nrf52833", context = "nrf52840"))]
pub type SensorI2c = i2c::controller::TWISPI0;
#[cfg(context = "nrf5340")]
pub type SensorI2c = i2c::controller::SERIAL0;
#[cfg(all(context = "nrf", not(context = "microbit-v2")))]
riot_rs::define_peripherals!(BusPeripherals {
    i2c0_sda: P0_03,
    i2c0_scl: P0_04,
});

#[cfg(all(context = "nrf", not(context = "microbit-v2")))]
riot_rs::define_peripherals!(ACCEL_IntPeripherals { i2c_int_int: P0_31 });

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(BusPeripherals {
    i2c0_sda: P0_16,
    i2c0_scl: P0_08,
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(ACCEL_IntPeripherals { i2c_int_int: P0_25 });

riot_rs::group_peripherals!(ACCEL_Peripherals {
    int: ACCEL_IntPeripherals,
    p: riot_rs_builtin_sensors::lsm303agr::Peripherals,
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(LedPeripherals {
    led_col1: P0_28,
    led_row1: P0_21,
});
