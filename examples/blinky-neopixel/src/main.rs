#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

mod pins;

use ariel_os::{
    hal,
    spi::main::{highest_freq_in, Kilohertz},
    time::{Duration, Timer},
};
use smart_leds_trait::{SmartLedsWriteAsync, RGB8};

const PIXEL_COUNT: usize = 1;

#[ariel_os::task(autostart, peripherals)]
async fn blinky(peripherals: pins::Peripherals) {
    let mut spi_config = hal::spi::main::Config::default();
    spi_config.frequency = const { highest_freq_in(Kilohertz::kHz(3000)..=Kilohertz::kHz(3500)) };

    let spi_bus = pins::Spi::new(
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        spi_config,
    );
    let mut ws2812 =
        ws2812_async::Ws2812::<_, ws2812_async::Grb, { 12 * PIXEL_COUNT }>::new(spi_bus);

    loop {
        let pixels = core::iter::once(RGB8::new(20, 0, 0));
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;

        let pixels = core::iter::once(RGB8::new(0, 20, 0));
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;

        let pixels = core::iter::once(RGB8::new(0, 0, 20));
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;

        let pixels = core::iter::once(RGB8::new(20, 20, 20));
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
