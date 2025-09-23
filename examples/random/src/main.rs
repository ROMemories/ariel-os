#![no_main]
#![no_std]

use ariel_os::debug::log::*;

#[ariel_os::task(autostart)]
async fn main() {
    for _ in 0..10 {
        let value = getrandom::u32().unwrap();
        info!("The random value of this round is {}.", value);
    }
}
