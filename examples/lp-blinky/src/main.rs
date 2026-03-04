#![no_main]
#![no_std]

use ariel_os_boards::pins;

use ariel_os::{
    gpio::{Level, Output},
    hal::peripherals,
    time::{Duration, Timer},
};

ariel_os::hal::group_peripherals!(Peripherals {
    leds: pins::LedPeripherals,
    rtc: RtcPeripherals,
});

ariel_os::hal::define_peripherals!(RtcPeripherals { rtc: RTC });

#[ariel_os::task(autostart, peripherals)]
async fn lp_blinky(peripherals: Peripherals) {
    let delay = Duration::from_millis(500);

    let mut led0 = Output::new(peripherals.leds.led0, Level::Low);
    led0.set_high();
    Timer::after(delay).await;
    led0.set_low();

    let rtc_config = embassy_stm32::rtc::RtcConfig::default();
    let rtc = embassy_stm32::rtc::Rtc::new(peripherals.rtc.rtc, rtc_config);
    critical_section::with(|cs| {
        rtc.start_wakeup_alarm(delay, cs);
    });

    ariel_os::power::enter_shutdown_mode()
}
