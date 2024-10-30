use riot_rs::arch::peripherals;

#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
riot_rs::define_peripherals!(BUTTON_1Peripherals { p: P0_11 });
#[cfg(all(feature = "button-readings", context = "st-nucleo-h755zi-q"))]
riot_rs::define_peripherals!(BUTTON_1Peripherals { p: PC13 });

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(AccelInterruptsPeripherals { int1: P0_22 });
#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(BusPeripherals {
    i2c0_sda: P0_03,
    i2c0_scl: P0_04,
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(BusPeripherals {
    i2c0_sda: P0_16,
    i2c0_scl: P0_08,
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(ACCEL_IntPeripherals { accel_int1: P0_16 });
#[cfg(context = "microbit-v2")]
riot_rs::group_peripherals!(ACCEL_Peripherals {
    int: ACCEL_IntPeripherals,
    p: riot_rs_builtin_sensors::lsm303agr::Peripherals,
});
