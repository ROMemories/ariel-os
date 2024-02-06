use embassy_nrf::peripherals;
use riot_rs::define_peripherals;

#[cfg(builder = "nrf52840dk")]
define_peripherals!(Leds {
    led1: P0_13,
    led2: P0_14,
    led3: P0_15,
    led4: P0_16,
});
