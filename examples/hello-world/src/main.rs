#![no_main]
#![no_std]

use ariel_os::{
    debug::{ExitCode, exit},
    hal::peripherals,
    log::*,
};

#[cfg(context = "esp")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin: GPIO0,
    led: GPIO1,
});

#[cfg(context = "stm32u083c-dk")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin: PC2,
    led: PC13,
});

#[ariel_os::task(autostart, peripherals)]
async fn main(mut p: Peripherals) {
    let mut led = ariel_os::gpio::Output::new(p.led, ariel_os::gpio::Level::High);

    // .build_with_interrupt()
    //     .unwrap();

    ariel_os::time::Timer::after_secs(2).await;

    loop {
        info!("Hello World!");

        for _ in 0..2 {
            led.toggle();
            ariel_os::time::Timer::after_millis(200).await;
            led.toggle();
            ariel_os::time::Timer::after_millis(200).await;
        }

        let mut wakeup = ariel_os::power::StopWakeupInterrupts::default();
        let mut input = ariel_os::gpio::Input::new(p.pin.reborrow(), ariel_os::gpio::Pull::Up);
        wakeup.gpio = Some((input, ariel_os::power::GpioWakeupTrigger::Low));
        ariel_os::power::enter_stop_mode(wakeup);
    }

    exit(ExitCode::SUCCESS);
}
