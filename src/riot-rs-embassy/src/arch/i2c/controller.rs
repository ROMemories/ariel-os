//! Architecture- and MCU-specific types for I2C.
//!
//! This module provides a driver for each I2C peripheral, the driver name being the same as the
//! peripheral; see the tests and examples to learn how to instantiate them.
//! These driver instances are meant to be shared between tasks using
//! [`I2cDevice`](crate::i2c::controller::I2cDevice).

/// Peripheral-agnostic I2C driver implementing [`embedded_hal_async::i2c::I2c`].
///
/// This type is not meant to be instantiated directly; instead instantiate a peripheral-specific
/// driver provided by this module.
// NOTE: we keep this type public because it may still required in user-written type signatures.
pub enum I2c {
    // Make the docs show that this enum has variants, but do not show any because they are
    // MCU-specific.
    #[doc(hidden)]
    Hidden,
}

impl embedded_hal_async::i2c::ErrorType for I2c {
    type Error = riot_rs_embassy_common::i2c::controller::Error;
}

impl riot_rs_embassy_common::reexports::embedded_hal_async::i2c::I2c for I2c {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        unimplemented!();
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        unimplemented!();
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        unimplemented!();
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [riot_rs_embassy_common::i2c::controller::Operation<'_>],
    ) -> Result<(), Self::Error> {
        unimplemented!();
    }
}

/// MCU-specific I2C bus frequency.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Frequency {
    /// Standard mode.
    _100k,
    /// Fast mode.
    _400k,
    #[doc(hidden)]
    Hidden,
}

impl Frequency {
    pub const fn first() -> Self {
        Self::_100k
    }

    pub const fn last() -> Self {
        Self::_400k
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::_100k => Some(Self::_400k),
            Self::_400k => None,
            Self::Hidden => unreachable!(),
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::_100k => None,
            Self::_400k => Some(Self::_100k),
            Self::Hidden => unreachable!(),
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::_100k => 100,
            Self::_400k => 400,
            Self::Hidden => unreachable!(),
        }
    }
}
