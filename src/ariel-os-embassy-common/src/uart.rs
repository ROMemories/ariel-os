//! Provides HAL-agnostic UART-related types.

/// Common UART baud rates.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Baud<A> {
    /// HAL-specific baud rate.
    Hal(A),
    /// 2400 bauds.
    _2400,
    /// 4800 bauds.
    _4800,
    /// 9600 bauds.
    _9600,
    /// 19200 bauds.
    _19200,
    /// 38400 bauds.
    _38400,
    /// 57600 bauds.
    _57600,
    /// 115200 bauds.
    _115200,
}

impl<A> From<Baud<A>> for u32
where
    u32: From<A>,
{
    fn from(b: Baud<A>) -> u32 {
        match b {
            Baud::Hal(hal) => hal.into(),
            Baud::_2400 => 2400,
            Baud::_4800 => 4800,
            Baud::_9600 => 9600,
            Baud::_19200 => 19200,
            Baud::_38400 => 38400,
            Baud::_57600 => 57600,
            Baud::_115200 => 115_200,
        }
    }
}

// #[cfg(feature = "defmt")]
// impl<A> defmt::Format for Baud<A> {
//     fn format(&self, f: defmt::Formatter<'_>) {
//         use defmt::write;
//         write!(f, "{=u32}", Into::<u32>::into((*self).clone()));
//     }
// }

/// Parity bit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity<A> {
    /// HAL-specific parity configuration.
    Hal(A),
    /// No parity bit.
    None,
    /// Even parity bit.
    Even,
}

// #[cfg(feature = "defmt")]
// impl<A> defmt::Format for Parity<A> {
//     fn format(&self, f: defmt::Formatter<'_>) {
//         A::from((*self).clone()).format(f)
//     }
// }

/// UART number of stop bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits<A> {
    /// HAL-specific stop bit configuration.
    Hal(A),
    /// One stop bit.
    Stop1,
}

// #[cfg(feature = "defmt")]
// impl<A> defmt::Format for StopBits<A> {
//     fn format(&self, f: defmt::Formatter<'_>) {
//         A::from((*self).clone()).format(f)
//     }
// }

/// UART number of data bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits<A> {
    /// HAL-specific number of data bits per character.
    Hal(A),
    /// 7 bits per character.
    Data7,
    /// 8 bits per character.
    Data8,
}

// #[cfg(feature = "defmt")]
// impl<A> defmt::Format for DataBits<A> {
//     fn format(&self, f: defmt::Formatter<'_>) {
//         A::from((*self).clone()).format(f)
//     }
// }

#[doc(hidden)]
#[macro_export]
macro_rules! impl_async_uart_bufread_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        impl embedded_io_async::BufRead for $driver_enum<'_> {
            async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::BufRead::fill_buf(&mut uart.uart).await, )*
                }
            }

            fn consume(&mut self, amt: usize) {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::BufRead::consume(&mut uart.uart, amt), )*
                }
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_async_uart_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        impl embedded_io_async::Read for $driver_enum<'_> {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::Read::read(&mut uart.uart, buf).await, )*
                }
            }
        }


        impl embedded_io_async::ReadReady for $driver_enum<'_> {
            fn read_ready(&mut self) -> Result<bool, Self::Error> {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::ReadReady::read_ready(&mut uart.uart), )*
                }
            }
        }

        impl embedded_io_async::Write for $driver_enum<'_> {
            async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::Write::write(&mut uart.uart, buf).await, )*
                }
            }
            async fn flush(&mut self) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::Write::flush(&mut uart.uart).await, )*
                }
            }
            async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(uart) => embedded_io_async::Write::write_all(&mut uart.uart, buf).await, )*
                }
            }
        }
    }
}
