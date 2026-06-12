#![no_main]
#![no_std]

use ariel_os::{
    debug::{ExitCode, exit},
    hal::peripherals,
    log::*,
};

ariel_os::hal::define_peripherals!(Peripherals {
    pin: GPIO_0,
    led: GPIO_1,
});

#[ariel_os::task(autostart, peripherals)]
async fn main(p: Peripherals) {
    let mut led = ariel_os::gpio::Output::new(p.led, ariel_os::gpio::Level::High);

    ariel_os::time::Timer::after_secs(2).await;

    loop {
        info!("Hello World!");

        led.toggle();
        ariel_os::time::Timer::after_millis(200).await;
        led.toggle();
        ariel_os::time::Timer::after_millis(200).await;

        // ariel_os::power::enter_stop_mode();
    }

    exit(ExitCode::SUCCESS);
}
