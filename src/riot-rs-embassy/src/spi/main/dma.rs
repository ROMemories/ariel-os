//! Provides support for the SPI communication bus in main mode, with DMA support.
//!
//! As setting up DMA may incur some overhead, it is most suitable for data transfers.

// FIXME: expand the doc comment to explain when to use it

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as InnerSpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::{arch, gpio};

/// An SPI driver implementing [`embedded_hal_async::spi::SpiDevice`].
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

