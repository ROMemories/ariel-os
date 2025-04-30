#![no_main]
#![no_std]

use ariel_os::debug::log::*;

#[ariel_os::task(autostart)]
async fn main() {
    // trace!("-- trace log level enabled");
    // debug!("-- debug log level enabled");
    // info!("-- info log level enabled");
    // warn!("-- warn log level enabled");
    // error!("-- error log level enabled (just testing)");

    loop {
        let test = core::hint::black_box(1);
        ariel_os::println!("Test: {}", 42 + test);
        ariel_os::time::Timer::after_millis(1000).await;
    }
}
