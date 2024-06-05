use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as InnerSpiDevice;
use embassy_rp::{
    dma, gpio, peripherals,
    spi::{Async, ClkPin, MisoPin, MosiPin, Phase, Polarity, Spi as InnerSpi},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

// TODO: factor this out across archs?
pub type SpiDevice = InnerSpiDevice<'static, CriticalSectionRawMutex, Spi, gpio::Output<'static>>;

#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::M1,
            mode: Mode::Mode0,
        }
    }
}

// Possible values are copied from embassy-nrf
// TODO: check how well this matches the RP2040 capabilities
#[repr(u32)]
pub enum Frequency {
    K125 = 125_000,
    K250 = 250_000,
    K500 = 500_00,
    M1 = 1_000_000,
    M2 = 2_000_000,
    M4 = 4_000_000,
    M8 = 8_000_000,
    M16 = 16_000_000,
    M32 = 32_000_000,
}

pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

// https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
impl From<Mode> for (Polarity, Phase) {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Mode0 => (Polarity::IdleLow, Phase::CaptureOnFirstTransition),
            Mode::Mode1 => (Polarity::IdleLow, Phase::CaptureOnSecondTransition),
            Mode::Mode2 => (Polarity::IdleHigh, Phase::CaptureOnFirstTransition),
            Mode::Mode3 => (Polarity::IdleHigh, Phase::CaptureOnSecondTransition),
        }
    }
}

macro_rules! define_spi_driver {
    ($peripheral:ident) => {
        paste::paste! {
            pub struct [<Spi $peripheral>] {
                spi: InnerSpi<'static, peripherals::$peripheral, Async>,
            }

            impl [<Spi $peripheral>] {
                #[must_use]
                pub fn new(
                    spi_peripheral: peripherals::$peripheral,
                    sck_pin: impl ClkPin<peripherals::$peripheral>,
                    miso_pin: impl MisoPin<peripherals::$peripheral>,
                    mosi_pin: impl MosiPin<peripherals::$peripheral>,
                    tx_dma: impl dma::Channel,
                    rx_dma: impl dma::Channel,
                    config: Config,
                ) -> Self {
                    let (pol, phase) = config.mode.into();

                    let mut spi_config = embassy_rp::spi::Config::default();
                    spi_config.frequency = config.frequency as u32;
                    spi_config.polarity = pol;
                    spi_config.phase = phase;

                    // The order of MOSI/MISO pins is inverted.
                    let spi = InnerSpi::new(
                        spi_peripheral,
                        sck_pin,
                        mosi_pin,
                        miso_pin,
                        tx_dma,
                        rx_dma,
                        spi_config,
                    );

                    Self { spi }
                }
            }

            impl embedded_hal_async::spi::SpiBus for [<Spi $peripheral>] {
                async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                    self.spi.read(words).await
                }

                async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
                    self.spi.write(data).await
                }

                async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Self::Error> {
                    self.spi.transfer(rx, tx).await
                }

                async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                    self.spi.transfer_in_place(words).await
                }

                async fn flush(&mut self) -> Result<(), Self::Error> {
                    self.spi.flush()
                }
            }

            impl embedded_hal_async::spi::ErrorType for [<Spi $peripheral>] {
                type Error = embassy_rp::spi::Error;
            }
        }
    };
}

define_spi_driver!(SPI0);
define_spi_driver!(SPI1);

// TODO: codegen this
pub enum Spi {
    SPI0(SpiSPI0),
    SPI1(SpiSPI1),
}

impl embedded_hal_async::spi::SpiBus for Spi {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        match self {
            Self::SPI0(spi) => spi.spi.read(words).await,
            Self::SPI1(spi) => spi.spi.read(words).await,
        }
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        match self {
            Self::SPI0(spi) => spi.spi.write(data).await,
            Self::SPI1(spi) => spi.spi.write(data).await,
        }
    }

    async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Self::Error> {
        match self {
            Self::SPI0(spi) => spi.spi.transfer(rx, tx).await,
            Self::SPI1(spi) => spi.spi.transfer(rx, tx).await,
        }
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        match self {
            Self::SPI0(spi) => spi.spi.transfer_in_place(words).await,
            Self::SPI1(spi) => spi.spi.transfer_in_place(words).await,
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        match self {
            Self::SPI0(spi) => spi.spi.flush(),
            Self::SPI1(spi) => spi.spi.flush(),
        }
    }
}

impl embedded_hal_async::spi::ErrorType for Spi {
    type Error = embassy_rp::spi::Error;
}
