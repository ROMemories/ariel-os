#![no_main]
#![no_std]

use ariel_os::{gpio::Pull, log::*};

mod pins {
    use ariel_os::hal::peripherals;

    #[cfg(not(any(context = "esp", context = "rp")))]
    pub use ariel_os_boards::pins::*;

    #[cfg(context = "esp")]
    ariel_os::hal::define_peripherals!(LedPeripherals { led0: GPIO1 });

    #[cfg(context = "esp")]
    ariel_os::hal::define_peripherals!(ButtonPeripherals { button0: GPIO0 });

    #[cfg(context = "rp")]
    ariel_os::hal::define_peripherals!(LedPeripherals { led0: PIN_1 });

    #[cfg(context = "rp")]
    ariel_os::hal::define_peripherals!(ButtonPeripherals { button0: PIN_0 });
}

ariel_os::hal::group_peripherals!(Peripherals {
    leds: pins::LedPeripherals,
    buttons: pins::ButtonPeripherals,
});

#[ariel_os::task(autostart, peripherals)]
async fn main(mut p: Peripherals) {
    let mut led = ariel_os::gpio::Output::new(p.leds.led0, ariel_os::gpio::Level::Low);

    // Makes for easier recovery of the MCU for flashing.
    ariel_os::time::Timer::after_secs(2).await;

    #[allow(unused_variables)]
    let pull = Pull::Up;
    #[cfg(context = "st-nucleo-h755zi-q")]
    let pull = Pull::None;

    loop {
        info!("Hello World!");

        for _ in 0..2 {
            led.toggle();
            ariel_os::time::Timer::after_millis(200).await;
            led.toggle();
            ariel_os::time::Timer::after_millis(200).await;
        }

        let mut wakeup = ariel_os::power::stop_mode::WakeupTriggers::default();
        wakeup.gpio = Some(ariel_os::power::stop_mode::GpioWakeupTrigger::new(
            p.buttons.button0.reborrow(),
            pull,
            ariel_os::power::stop_mode::GpioWakeupTriggerEvent::Low,
        ));
        ariel_os::power::stop_mode::enter(wakeup);
    }
}
