//! Provides architecture-agnostic SPI-related types, for main mode.

// FIXME: rename this to Bitrate and use bps instead?
/// SPI bus frequency.
#[derive(Copy, Clone)]
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_spi_from_frequency {
    () => {
        impl From<riot_rs_shared_types::spi::main::Frequency> for Frequency {
            fn from(freq: riot_rs_shared_types::spi::main::Frequency) -> Self {
                match freq {
                    riot_rs_shared_types::spi::main::Frequency::_125k => Frequency::_125k,
                    riot_rs_shared_types::spi::main::Frequency::_250k => Frequency::_250k,
                    riot_rs_shared_types::spi::main::Frequency::_500k => Frequency::_500k,
                    riot_rs_shared_types::spi::main::Frequency::_1M => Frequency::_1M,
                    riot_rs_shared_types::spi::main::Frequency::_2M => Frequency::_2M,
                    riot_rs_shared_types::spi::main::Frequency::_4M => Frequency::_4M,
                    riot_rs_shared_types::spi::main::Frequency::_8M => Frequency::_8M,
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_spi_frequency_const_functions_32M {
    () => {
        impl Frequency {
            pub const fn first() -> Self {
                Self::_125k
            }

            pub const fn last() -> Self {
                Self::_32M
            }

            pub const fn next(self) -> Option<Self> {
                match self {
                    Self::_125k => Some(Self::_250k),
                    Self::_250k => Some(Self::_500k),
                    Self::_500k => Some(Self::_1M),
                    Self::_1M => Some(Self::_2M),
                    Self::_2M => Some(Self::_4M),
                    Self::_4M => Some(Self::_8M),
                    Self::_8M => Some(Self::_16M),
                    Self::_16M => Some(Self::_32M),
                    Self::_32M => None,
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
                    Self::_16M => Some(Self::_8M),
                    Self::_32M => Some(Self::_16M),
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
                    Self::_16M => 16_000,
                    Self::_32M => 32_000,
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_async_spibus_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        // The `SpiBus` trait represents exclusive ownership over the whole bus.
        impl embedded_hal_async::spi::SpiBus for $driver_enum {
            async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.read(words).await, )*
                }
            }

            async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.write(data).await, )*
                }
            }

            async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.transfer(rx, tx).await, )*
                }
            }

            async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.transfer_in_place(words).await, )*
                }
            }

            async fn flush(&mut self) -> Result<(), Self::Error> {
                use embedded_hal_async::spi::SpiBus;
                match self {
                    $( Self::$peripheral(spi) => SpiBus::<u8>::flush(&mut spi.spim).await, )*
                }
            }
        }
    }
}
