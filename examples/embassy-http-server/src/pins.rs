use riot_rs::define_peripherals;

use riot_rs::embassy::arch::peripherals;

define_peripherals!(Peripherals {
    #[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
    buttons: Buttons {
        btn1: P0_11,
        btn2: P0_12,
        btn3: P0_24,
        btn4: P0_25,
    }
});
