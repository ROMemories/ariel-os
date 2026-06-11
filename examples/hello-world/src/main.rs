#![no_main]
#![no_std]

use ariel_os::{
    debug::{ExitCode, exit},
    log::*,
    hal::peripherals,
};

ariel_os::hal::define_peripherals!(Peripherals {
    pin: PIN_0,
});

#[ariel_os::task(autostart, peripherals)]
async fn main(p: Peripherals) {
    // let mut pin = embassy_rp::gpio::Input::new(p.pin, embassy_rp::gpio::Pull::Up);
    // pin.dormant_wake(embassy_rp::gpio::DormantWakeConfig {
    //     edge_low: true,
    //     ..Default::default()
    // });

    // loop {
    info!("Hello World!");

    ariel_os::time::Timer::after_secs(2).await;

    // ariel_os::power::enter_stop_mode();
    // }

    exit(ExitCode::SUCCESS);
}
