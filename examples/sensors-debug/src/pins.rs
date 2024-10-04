use riot_rs::arch::peripherals;

#[cfg(all(feature = "button-readings", context = "nrf52840dk"))]
riot_rs::define_peripherals!(BUTTON_1Peripherals { p: P0_11 });
#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(AccelInterruptsPeripherals { int1: P0_22 });
