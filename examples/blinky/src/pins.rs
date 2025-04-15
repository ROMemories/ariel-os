use ariel_os::hal::{i2c, peripherals};

#[cfg(context = "bbc-microbit-v2")]
ariel_os::hal::define_peripherals!(LedPeripherals {
    led_col1: P0_28,
    led: P0_21,
});

#[cfg(context = "dfrobot-firebeetle2-esp32-c6")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: GPIO15 });

#[cfg(context = "nordic-thingy-91-x-nrf9151")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: P0_29 });

#[cfg(context = "nrf52840dk")]
ariel_os::hal::define_peripherals!(LedPeripherals {
    led: P0_13,
    uart0: UARTE0,
    // uart_rx: P0_00,
    // uart_tx: P0_01,
    i2c_sda: P0_26,
    i2c_scl: P0_27,
});

#[cfg(context = "nrf5340dk")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: P0_28 });

#[cfg(context = "nrf9160dk-nrf9160")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: P0_02 });

#[cfg(context = "particle-xenon")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: P1_12 });

#[cfg(any(context = "rpi-pico", context = "rpi-pico2"))]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PIN_25 });

#[cfg(all(context = "rp", not(any(context = "rpi-pico", context = "rpi-pico2"))))]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PIN_1 });

#[cfg(all(context = "esp", not(context = "dfrobot-firebeetle2-esp32-c6")))]
ariel_os::hal::define_peripherals!(LedPeripherals { led: GPIO0 });

#[cfg(context = "st-nucleo-c031c6")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PA5 });

#[cfg(context = "st-nucleo-f401re")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PA5 });

#[cfg(context = "st-nucleo-h755zi-q")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PB0 });

#[cfg(context = "st-nucleo-wb55")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PB5 });

#[cfg(context = "st-nucleo-wba55")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PB4 });

#[cfg(context = "stm32u083c-dk")]
ariel_os::hal::define_peripherals!(LedPeripherals { led: PA5 });

#[cfg(any(context = "nrf52833", context = "nrf52840"))]
pub type SensorI2c = i2c::controller::TWISPI0;
#[cfg(any(context = "nrf5340", context = "nrf91"))]
pub type SensorI2c = i2c::controller::SERIAL0;
#[cfg(all(
    context = "nrf",
    not(any(context = "bbc-microbit-v2", context = "nordic-thingy-91-x-nrf9151"))
))]
ariel_os::hal::define_peripherals!(Peripherals {
    i2c_sda: P0_00,
    i2c_scl: P0_01,
});
