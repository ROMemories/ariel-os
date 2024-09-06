#[doc(alias = "master")]
pub mod main;

use riot_rs_shared_types::spi::{BitOrder, Mode};

fn from_mode(mode: Mode) -> embassy_stm32::spi::Mode {
    match mode {
        Mode::Mode0 => embassy_stm32::spi::MODE_0,
        Mode::Mode1 => embassy_stm32::spi::MODE_1,
        Mode::Mode2 => embassy_stm32::spi::MODE_2,
        Mode::Mode3 => embassy_stm32::spi::MODE_3,
    }
}

fn from_bit_order(bit_order: BitOrder) -> embassy_stm32::spi::BitOrder {
    match bit_order {
        BitOrder::MsbFirst => embassy_stm32::spi::BitOrder::MsbFirst,
        BitOrder::LsbFirst => embassy_stm32::spi::BitOrder::LsbFirst,
    }
}
