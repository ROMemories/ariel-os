#![no_main]
#![no_std]

mod pins;

use ariel_os::{
    hal,
    spi::main::{Kilohertz, highest_freq_in},
    time::{Duration, Timer},
};
use smart_leds_trait::{RGB8, SmartLedsWriteAsync};

const PIXEL_COUNT: usize = cfg_select! {
    context = "espressif-esp32-c6-devkitc-1" => 1,
    context = "unihiker-k10" => 3,
    _ => panic!("board not supported"),
};

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
    let mut ws2812 = ws2812_async::Ws2812::<_, ws2812_async::Grb, PIXEL_COUNT>::new(spi_bus);

    // Reset the closest LED.
    let pixels = core::iter::empty::<RGB8>();
    ws2812.write(pixels).await.unwrap();

    loop {
        let pixels = core::iter::repeat(RGB8::new(20, 0, 0)).take(PIXEL_COUNT);
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;

        let pixels = core::iter::repeat(RGB8::new(0, 20, 0)).take(PIXEL_COUNT);
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;

        let pixels = core::iter::repeat(RGB8::new(0, 0, 20)).take(PIXEL_COUNT);
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;

        let pixels = core::iter::repeat(RGB8::new(20, 20, 20)).take(PIXEL_COUNT);
        ws2812.write(pixels).await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
