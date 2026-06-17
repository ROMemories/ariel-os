#![no_main]
#![no_std]

use ariel_os::{
    debug::{ExitCode, exit},
    hal::peripherals,
    log::*,
};

ariel_os::hal::define_peripherals!(Peripherals {
    pin: GPIO0,
    led: GPIO1,
    rtc: LPWR,
});

#[ariel_os::task(autostart, peripherals)]
async fn main(mut p: Peripherals) {
    let mut led = ariel_os::gpio::Output::new(p.led, ariel_os::gpio::Level::High);
    // let mut rtc = esp_hal::rtc_cntl::Rtc::new(p.rtc);

    ariel_os::time::Timer::after_secs(2).await;

    // let var = ;
    // let config = esp_hal::gpio::InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
    // let mut pin = esp_hal::gpio::Input::new(p.pin, config);

    loop {
        info!("Hello World!");

        led.toggle();
        ariel_os::time::Timer::after_millis(200).await;
        led.toggle();
        ariel_os::time::Timer::after_millis(200).await;

        // let wakeup_pins: &mut [(&mut dyn esp_hal::gpio::RtcPinWithResistors, _)] =
        //     &mut [(&mut p.pin, esp_hal::rtc_cntl::sleep::WakeupLevel::Low)];

        // let ext1 = esp_hal::rtc_cntl::sleep::GpioWakeupSource::new();
        // rtc.sleep_light(&[&ext1]);

        let mut input = ariel_os::gpio::Input::new(p.pin.reborrow(), ariel_os::gpio::Pull::Up);
        ariel_os::power::enter_stop_mode(input);
    }

    exit(ExitCode::SUCCESS);
}
