//! The input and output GPIO pins need to be connected.

#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

mod pins;

use ariel_os::{
    debug::log::info,
    gpio::{Input, Level, Output, Pull},
    time::{Duration, Instant, Timer},
};
use embassy_futures::yield_now;

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: pins::InputPeripheral) {
    let mut input = Input::builder(peripherals.input, Pull::Up)
        .build_with_interrupt()
        .unwrap();

    // Wait for the other task to start
    input.wait_for_any_edge().await;

    let now = Instant::now();
    for _ in 0..1000 {
        input.wait_for_any_edge().await;
    }
    info!("{}", now.elapsed());
}

#[ariel_os::task(autostart, peripherals)]
async fn toggle(peripherals: pins::OutputPeripheral) {
    let mut output = Output::new(peripherals.output, Level::High);

    Timer::after(Duration::from_millis(10)).await;

    loop {
        output.toggle();
        yield_now().await;
    }
}
