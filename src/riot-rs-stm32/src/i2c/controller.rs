use embassy_stm32::{
    bind_interrupts,
    i2c::{
        ErrorInterruptHandler, EventInterruptHandler, I2c as InnerI2c, RxDma, SclPin, SdaPin, TxDma,
    },
    mode::Async,
    peripherals,
    time::Hertz,
    Peripheral,
};
use riot_rs_macros::call_with_stm32_peripheral_list;
use riot_rs_shared_types::impl_async_i2c_for_driver_enum;

/// I2C bus configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::UpTo100k(100),
            sda_pullup: false,
            scl_pullup: false,
        }
    }
}

/// I2C bus frequency.
// FIXME(embassy): fast mode plus is supported by hardware but requires additional configuration
// that Embassy does not seem to currently provide.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    /// Standard mode.
    UpTo100k(u32), // FIXME: use a range integer?
    /// Fast mode.
    UpTo400k(u32), // FIXME: use a range integer?
}

impl Frequency {
    pub const fn first() -> Self {
        Self::UpTo100k(1)
    }

    pub const fn last() -> Self {
        Self::UpTo400k(400)
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::UpTo100k(f) => {
                if f < 100 {
                    // NOTE(no-overflow): `f` is small enough due to if condition
                    Some(Self::UpTo100k(f + 1))
                } else {
                    Some(Self::UpTo400k(self.khz() + 1))
                }
            }
            Self::UpTo400k(f) => {
                if f < 400 {
                    // NOTE(no-overflow): `f` is small enough due to if condition
                    Some(Self::UpTo400k(f + 1))
                } else {
                    None
                }
            }
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::UpTo100k(f) => {
                if f > 1 {
                    // NOTE(no-overflow): `f` is large enough due to if condition
                    Some(Self::UpTo100k(f - 1))
                } else {
                    None
                }
            }
            Self::UpTo400k(f) => {
                if f > 100 + 1 {
                    // NOTE(no-overflow): `f` is large enough due to if condition
                    Some(Self::UpTo400k(f - 1))
                } else {
                    Some(Self::UpTo100k(self.khz() - 1))
                }
            }
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::UpTo100k(f) | Self::UpTo400k(f) => f,
        }
    }
}

riot_rs_shared_types::impl_i2c_from_frequency_up_to!();

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::UpTo100k(f) | Frequency::UpTo400k(f) => Hertz::khz(f),
        }
    }
}

pub(crate) fn init(peripherals: &mut crate::OptionalPeripherals) {
    // This macro has to be defined in this function so that the `peripherals` variables exists.
    macro_rules! take_all_i2c_peripherals {
        ($peripherals:ident, $( $peripheral:ident ),*) => {
            $(
                let _ = peripherals.$peripheral.take().unwrap();
            )*
        }
    }

    // Take all I2c peripherals and do nothing with them.
    call_with_stm32_peripheral_list!(take_all_i2c_peripherals!, I2c, Peripherals);
}

macro_rules! define_i2c_drivers {
    ($( $ev_interrupt:ident + $er_interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            // NOTE(arch): this is not required on this architecture, as the inner I2C type is
            // not generic over the I2C peripheral, and is only done for consistency with
            // other architectures.
            pub struct $peripheral {
                twim: InnerI2c<'static, Async>
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sda_pin: impl Peripheral<P: SdaPin<peripherals::$peripheral>> + 'static,
                    scl_pin: impl Peripheral<P: SclPin<peripherals::$peripheral>> + 'static,
                    tx_dma: impl Peripheral<P: TxDma<peripherals::$peripheral>> + 'static,
                    rx_dma: impl Peripheral<P: RxDma<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> I2c {
                    let mut i2c_config = embassy_stm32::i2c::Config::default();
                    i2c_config.sda_pullup = config.sda_pullup;
                    i2c_config.scl_pullup = config.scl_pullup;
                    i2c_config.timeout = riot_rs_shared_types::i2c::controller::I2C_TIMEOUT;

                    bind_interrupts!(
                        struct Irqs {
                            $ev_interrupt => EventInterruptHandler<peripherals::$peripheral>;
                            $er_interrupt => ErrorInterruptHandler<peripherals::$peripheral>;
                        }
                    );

                    // Make this struct a compile-time-enforced singleton: having multiple statics
                    // defined with the same name would result in a compile-time error.
                    paste::paste! {
                        #[allow(dead_code)]
                        static [<PREVENT_MULTIPLE_ $peripheral>]: () = ();
                    }

                    // FIXME(safety): enforce that the init code indeed has run
                    // SAFETY: this struct being a singleton prevents us from stealing the
                    // peripheral multiple times.
                    let twim_peripheral = unsafe { peripherals::$peripheral::steal() };

                    let frequency = config.frequency;
                    let i2c = InnerI2c::new(
                        twim_peripheral,
                        scl_pin,
                        sda_pin,
                        Irqs,
                        tx_dma,
                        rx_dma,
                        frequency.into(),
                        i2c_config,
                    );

                    I2c::$peripheral(Self { twim: i2c })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $( $peripheral($peripheral), )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = riot_rs_shared_types::i2c::controller::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
    }
}

// We cannot impl From because both types are external to this crate.
fn from_error(err: embassy_stm32::i2c::Error) -> riot_rs_shared_types::i2c::controller::Error {
    use embassy_stm32::i2c::Error::*;

    use riot_rs_shared_types::i2c::controller::{Error, NoAcknowledgeSource};

    match err {
        Bus => Error::Bus,
        Arbitration => Error::ArbitrationLoss,
        Nack => Error::NoAcknowledge(NoAcknowledgeSource::Unknown),
        Timeout => Error::Timeout,
        Crc => Error::Other,
        Overrun => Error::Overrun,
        ZeroLengthTransfer => Error::Other,
    }
}

// Define a driver per peripheral
call_with_stm32_peripheral_list!(define_i2c_drivers!, I2c, PeripheralsAndInterrupts);
