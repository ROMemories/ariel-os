use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    spim::{InterruptHandler, Spim, MODE_0, MODE_1, MODE_2, MODE_3},
};

pub use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
pub use embassy_nrf::spim::Frequency;

#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
    pub bit_order: BitOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::M1,
            mode: Mode::Mode0,
            bit_order: BitOrder::MsbFirst,
        }
    }
}

pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

// https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
impl From<Mode> for embassy_nrf::spim::Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Mode0 => MODE_0,
            Mode::Mode1 => MODE_1,
            Mode::Mode2 => MODE_2,
            Mode::Mode3 => MODE_3,
        }
    }
}

pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

impl From<BitOrder> for embassy_nrf::spim::BitOrder {
    fn from(bit_order: BitOrder) -> Self {
        match bit_order {
            BitOrder::MsbFirst => embassy_nrf::spim::BitOrder::MSB_FIRST,
            BitOrder::LsbFirst => embassy_nrf::spim::BitOrder::LSB_FIRST,
        }
    }
}

bind_interrupts!(
    struct Irqs {
        SPIM2_SPIS2_SPI2 => InterruptHandler<peripherals::SPI2>;
    }
);

// FIXME: support other SPI peripherals as well
pub struct Spi {
    spim: Spim<'static, peripherals::SPI2>,
}

impl Spi {
    #[must_use]
    pub fn new(
        spim_peripheral: peripherals::SPI2,
        sck_pin: impl GpioPin,
        miso_pin: impl GpioPin,
        mosi_pin: impl GpioPin,
        config: Config,
    ) -> Self {
        let mut spi_config = embassy_nrf::spim::Config::default();
        spi_config.frequency = config.frequency;
        spi_config.mode = config.mode.into();
        spi_config.bit_order = config.bit_order.into();

        let spim = Spim::new(
            spim_peripheral,
            Irqs,
            sck_pin,
            miso_pin,
            mosi_pin,
            spi_config,
        );

        Self { spim }
    }
}

impl embedded_hal_async::spi::SpiBus for Spi {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spim.read(words).await
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.spim.write(data).await
    }

    async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Self::Error> {
        self.spim.transfer(rx, tx).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.spim.transfer_in_place(words).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.spim.flush().await
    }
}

impl embedded_hal_async::spi::ErrorType for Spi {
    type Error = embassy_nrf::spim::Error;
}
