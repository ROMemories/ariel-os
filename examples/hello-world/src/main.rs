#![no_main]
#![no_std]

use ariel_os::{
    debug::{ExitCode, exit},
    log::*,
};

#[ariel_os::task(autostart)]
async fn main() {
    info!("Hello World!");

    ariel_os::time::Timer::after_secs(2).await;
    ariel_os::power::enter_stop_mode();

    exit(ExitCode::SUCCESS);
}
