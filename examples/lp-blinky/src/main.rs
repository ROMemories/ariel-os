#![no_main]
#![no_std]

use ariel_os_boards::pins;

use ariel_os::{
    gpio::{Level, Output},
    hal::peripherals,
    time::{Duration, Timer},
};

#[ariel_os::config(rcc)]
const RCC_CONFIG: embassy_stm32::rcc::Config = {
    use embassy_stm32::rcc::*;

    let mut rcc = embassy_stm32::rcc::Config::new();

    rcc.ls = LsConfig {
        rtc: RtcClockSource::LSE,
        lsi: true, // TODO: consider turning it off
        lse: Some(LseConfig {
            frequency: embassy_stm32::time::Hertz(32768),
            mode: LseMode::Oscillator(LseDrive::MediumHigh),
        }),
    };
    rcc.hsi = false;
    rcc.sys = Sysclk::MSI; // Embassy currently does not support LSE as SYSCLK.
    rcc.msi = Some(MSIRange::RANGE100K);

    rcc
};

ariel_os::hal::group_peripherals!(Peripherals {
    leds: pins::LedPeripherals,
    rtc: RtcPeripherals,
});

ariel_os::hal::define_peripherals!(RtcPeripherals { rtc: RTC });

#[ariel_os::task(autostart, peripherals)]
async fn lp_blinky(peripherals: Peripherals) {
    let delay = Duration::from_millis(500);

    embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpr(true));

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
