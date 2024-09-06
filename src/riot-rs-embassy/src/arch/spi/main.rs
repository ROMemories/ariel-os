//! Architecture- and MCU-specific types for SPI.
//!
//! This module provides a driver for each SPI peripheral, the driver name being the same as the
//! peripheral; see the tests and examples to learn how to instantiate them.
//! These driver instances are meant to be shared between tasks using
//! [`SpiDevice`](crate::spi::main::SpiDevice).

use crate::arch;

/// Peripheral-agnostic SPI driver implementing [`embedded_hal_async::spi::SpiBus`].
///
/// This type is not meant to be instantiated directly; instead instantiate a peripheral-specific
/// driver provided by this module.
// NOTE: we keep this type public because it may still required in user-written type signatures.
pub enum Spi {
    // Make the docs show that this enum has variants, but do not show any because they are
    // MCU-specific.
    #[doc(hidden)]
    Hidden,
}

/// MCU-specific I2C bus frequency.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Frequency {
    /// 125 kHz.
    _125k,
    /// 250 kHz.
    _250k,
    /// 500 kHz.
    _500k,
    /// 1 MHz.
    _1M,
    /// 2 MHz.
    _2M,
    /// 4 MHz.
    _4M,
    /// 8 MHz.
    _8M,
    #[doc(hidden)]
    Hidden,
}

impl Frequency {
    pub const fn first() -> Self {
        Self::_125k
    }

    pub const fn last() -> Self {
        Self::_8M
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::_125k => Some(Self::_250k),
            Self::_250k => Some(Self::_500k),
            Self::_500k => Some(Self::_1M),
            Self::_1M => Some(Self::_2M),
            Self::_2M => Some(Self::_4M),
            Self::_4M => Some(Self::_8M),
            Self::_8M => None,
            Self::Hidden => unreachable!(),
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::_125k => None,
            Self::_250k => Some(Self::_125k),
            Self::_500k => Some(Self::_250k),
            Self::_1M => Some(Self::_500k),
            Self::_2M => Some(Self::_1M),
            Self::_4M => Some(Self::_2M),
            Self::_8M => Some(Self::_4M),
            Self::Hidden => unreachable!(),
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::_125k => 125,
            Self::_250k => 250,
            Self::_500k => 500,
            Self::_1M => 1000,
            Self::_2M => 2000,
            Self::_4M => 4000,
            Self::_8M => 8000,
            Self::Hidden => unreachable!(),
        }
    }
}

pub(crate) fn init(peripherals: &mut arch::OptionalPeripherals) {
    unimplemented!();
}
