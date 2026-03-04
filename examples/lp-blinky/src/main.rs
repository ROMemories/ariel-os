#![no_main]
#![no_std]

use ariel_os_boards::pins;

use ariel_os::{
    hal::peripherals,
    gpio::{Level, Output},
    time::Timer,
};

ariel_os::hal::group_peripherals!(Peripherals {
    leds: pins::LedPeripherals,
    rtc: RtcPeripherals,
});

ariel_os::hal::define_peripherals!(RtcPeripherals {
    rtc: RTC,
});

#[ariel_os::task(autostart, peripherals)]
async fn lp_blinky(peripherals: Peripherals) {
    let mut led0 = Output::new(peripherals.leds.led0, Level::Low);

    led0.set_high();
    Timer::after_millis(200).await;
    led0.set_low();

    // FIXME: set up RTC for an interrupt in 5 s
    let rtc = embassy_stm32::rtc::Rtc::new(peripherals.rtc.rtc);


    ariel_os::power::enter_shutdown_mode()
}
