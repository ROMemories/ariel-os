#![no_main]
#![no_std]

use ariel_os_boards::pins;

use ariel_os::{
    gpio::{Level, Output},
    time::Timer,
};

#[ariel_os::task(autostart, peripherals)]
async fn lp_blinky(peripherals: pins::LedPeripherals) {
    let mut led0 = Output::new(peripherals.led0, Level::Low);

    led0.set_high();
    Timer::after_millis(200).await;
    led0.set_low();

    // FIXME: set up clocks
    // FIXME: set up RTC for an interrupt in 5 s

    ariel_os::power::enter_shutdown_mode()
}
