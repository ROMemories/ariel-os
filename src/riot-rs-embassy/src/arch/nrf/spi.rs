use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    spim::{InterruptHandler, Spim},
};

pub use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
// FIXME: maybe we should provide our own config type, unified across archs
pub use embassy_nrf::spim::{BitOrder, Config, Frequency, Mode, MODE_0, MODE_1, MODE_2, MODE_3};

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
        let spim = Spim::new(spim_peripheral, Irqs, sck_pin, miso_pin, mosi_pin, config);

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
