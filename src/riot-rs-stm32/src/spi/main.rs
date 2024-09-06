use embassy_stm32::{
    gpio,
    mode::Async,
    peripherals,
    spi::{MisoPin, MosiPin, RxDma, SckPin, Spi as InnerSpi, TxDma},
    time::Hertz,
    Peripheral,
};
use riot_rs_macros::call_with_stm32_peripheral_list;
use riot_rs_shared_types::{
    impl_async_spibus_for_driver_enum,
    spi::{BitOrder, Mode},
};

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
    pub bit_order: BitOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::_1M,
            mode: Mode::Mode0,
            bit_order: BitOrder::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    _125k,
    _250k,
    _500k,
    _1M,
    _2M,
    _4M,
    _8M,
    _16M,
    _32M,
}

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::_125k => Hertz::khz(125),
            Frequency::_250k => Hertz::khz(250),
            Frequency::_500k => Hertz::khz(500),
            Frequency::_1M => Hertz::mhz(1),
            Frequency::_2M => Hertz::mhz(2),
            Frequency::_4M => Hertz::mhz(4),
            Frequency::_8M => Hertz::mhz(8),
            Frequency::_16M => Hertz::mhz(16),
            Frequency::_32M => Hertz::mhz(32),
        }
    }
}

riot_rs_shared_types::impl_spi_from_frequency!();
riot_rs_shared_types::impl_spi_frequency_const_functions_32M!();

pub(crate) fn init(peripherals: &mut crate::OptionalPeripherals) {
    // This macro has to be defined in this function so that the `peripherals` variables exists.
    macro_rules! take_all_spi_peripherals {
        ($peripherals:ident, $( $peripheral:ident ),*) => {
            $(
                let _ = peripherals.$peripheral.take().unwrap();
            )*
        }
    }

    // Take all SPI peripherals and do nothing with them.
    call_with_stm32_peripheral_list!(take_all_spi_peripherals!, Spi, Peripherals);
}

macro_rules! define_spi_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: InnerSpi<'static, Async>,
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: SckPin<peripherals::$peripheral>> + 'static,
                    miso_pin: impl Peripheral<P: MisoPin<peripherals::$peripheral>> + 'static,
                    mosi_pin: impl Peripheral<P: MosiPin<peripherals::$peripheral>> + 'static,
                    tx_dma: impl Peripheral<P: TxDma<peripherals::$peripheral>> + 'static,
                    rx_dma: impl Peripheral<P: RxDma<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> Spi {
                    let mut spi_config = embassy_stm32::spi::Config::default();
                    spi_config.frequency = config.frequency.into();
                    spi_config.mode = crate::spi::from_mode(config.mode);
                    spi_config.bit_order = crate::spi::from_bit_order(config.bit_order);
                    spi_config.miso_pull = gpio::Pull::None; // FIXME: ?

                    // Make this struct a compile-time-enforced singleton: having multiple statics
                    // defined with the same name would result in a compile-time error.
                    paste::paste! {
                        #[allow(dead_code)]
                        static [<PREVENT_MULTIPLE_ $peripheral>]: () = ();
                    }

                    // FIXME(safety): enforce that the init code indeed has run
                    // SAFETY: this struct being a singleton prevents us from stealing the
                    // peripheral multiple times.
                    let spim_peripheral = unsafe { peripherals::$peripheral::steal() };

                    // The order of MOSI/MISO pins is inverted.
                    let spim = InnerSpi::new(
                        spim_peripheral,
                        sck_pin,
                        mosi_pin,
                        miso_pin,
                        tx_dma,
                        rx_dma,
                        spi_config,
                    );

                    Spi::$peripheral(Self { spim })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum Spi {
            $( $peripheral($peripheral) ),*
        }

        impl embedded_hal_async::spi::ErrorType for Spi {
            type Error = embassy_stm32::spi::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// Define a driver per peripheral
call_with_stm32_peripheral_list!(define_spi_drivers!, Spi, PeripheralsAndInterrupts);
