#![no_main]
#![no_std]

use ariel_os_boards::pins;

use ariel_os::{
    gpio::{Input, Level, Output, Pull},
    time::Timer,
};

ariel_os::hal::group_peripherals!(Peripherals {
    leds: pins::LedPeripherals,
    buttons: pins::ButtonPeripherals,
});

#[ariel_os::task(autostart, peripherals)]
async fn blinky(peripherals: Peripherals) {
    let mut led0 = Output::new(peripherals.leds.led0, Level::Low);

    #[allow(unused_variables)]
    let pull = Pull::Up;
    #[cfg(context = "st-nucleo-h755zi-q")]
    let pull = Pull::None;

    let mut btn0 = Input::builder(peripherals.buttons.button0, pull)
        .build_with_interrupt()
        .unwrap();

    Timer::after_millis(100).await;

    loop {
        led0.toggle();
        Timer::after_millis(100).await;
        led0.toggle();
        Timer::after_millis(100).await;
        led0.toggle();
        Timer::after_millis(100).await;
        led0.toggle();
        Timer::after_millis(500).await;

        // embassy_nrf::pac::P0
        //     .pin_cnf(11)
        //     .modify(|w| w.set_sense(embassy_nrf::pac::gpio::vals::Sense::LOW));

        // FIXME: check whether pull resistors are still enabled in standby mode.
        // embassy_stm32::pac::PWR.cr4().modify(|w| w.set_wp1(true));
        // // FIXME: clear all the wake-up flags.
        // // Clear the wake-up flags.
        // embassy_stm32::pac::PWR.scr().modify(|w| w.set_cwuf1(true));
        // embassy_stm32::pac::PWR.cr3().modify(|w| w.set_ewup1(true));

        ariel_os::power::enter_stop_mode();
    }
}
