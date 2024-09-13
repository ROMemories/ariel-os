//! Provides support for the SPI communication bus in main mode.

pub mod dma;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice as InnerSpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::{arch, gpio};

pub use riot_rs_shared_types::spi::main::*;

/// An SPI driver implementing [`embedded_hal::spi::SpiDevice`].
///
/// Needs to be provided with an MCU-specific SPI driver tied to a specific SPI peripheral,
/// obtainable from the [`arch::spi::main`] module.
/// It also requires a [`gpio::Output`] for the chip-select (CS) signal.
///
/// See [`embedded_hal::spi`] to learn more about the distinction between an
/// [`SpiBus`](embedded_hal::spi::SpiBus) and an
/// [`SpiDevice`](embedded_hal::spi::SpiDevice).
// TODO: do we actually need a CriticalSectionRawMutex here?
pub type SpiDevice =
    InnerSpiDevice<'static, CriticalSectionRawMutex, arch::spi::main::Spi, gpio::Output>;

crate::define_highest_khz_freq_in!(arch::spi::main::Frequency);
