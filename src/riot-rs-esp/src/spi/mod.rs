#[doc(alias = "master")]
pub mod main;

use riot_rs_shared_types::spi::{BitOrder, Mode};

fn from_mode(mode: Mode) -> esp_hal::spi::SpiMode {
    match mode {
        Mode::Mode0 => esp_hal::spi::SpiMode::Mode0,
        Mode::Mode1 => esp_hal::spi::SpiMode::Mode1,
        Mode::Mode2 => esp_hal::spi::SpiMode::Mode2,
        Mode::Mode3 => esp_hal::spi::SpiMode::Mode3,
    }
}

fn from_bit_order(bit_order: BitOrder) -> esp_hal::spi::SpiBitOrder {
    match bit_order {
        BitOrder::MsbFirst => esp_hal::spi::SpiBitOrder::MSBFirst,
        BitOrder::LsbFirst => esp_hal::spi::SpiBitOrder::LSBFirst,
    }
}
