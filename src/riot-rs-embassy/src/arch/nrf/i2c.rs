use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    twim::{InterruptHandler, Twim},
};
use embedded_hal_async::i2c::{Operation, SevenBitAddress};

pub use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
pub use embassy_nrf::twim::Frequency;

#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
    pub sda_high_drive: bool,
    pub scl_high_drive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
            sda_pullup: false,
            scl_pullup: false,
            sda_high_drive: false,
            scl_high_drive: false,
        }
    }
}

// FIXME: does this prevent us from binding another interrupt handler to the same interrupt (e.g.,
// for SPI), elsewhere?
bind_interrupts!(
    struct Irqs {
        SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => InterruptHandler<peripherals::TWISPI0>;
    }
);

// FIXME: support other I2C peripherals as well
pub struct I2c {
    twim: Twim<'static, peripherals::TWISPI0>,
}

impl I2c {
    #[must_use]
    pub fn new(
        twim_peripheral: peripherals::TWISPI0,
        sda_pin: impl GpioPin,
        scl_pin: impl GpioPin,
        config: Config,
    ) -> Self {
        let mut twim_config = embassy_nrf::twim::Config::default();
        twim_config.frequency = config.frequency;
        twim_config.sda_pullup = config.sda_pullup;
        twim_config.scl_pullup = config.scl_pullup;
        twim_config.sda_high_drive = config.sda_high_drive;
        twim_config.scl_high_drive = config.scl_high_drive;

        let twim = Twim::new(twim_peripheral, Irqs, sda_pin, scl_pin, twim_config);

        Self { twim }
    }
}

impl embedded_hal_async::i2c::I2c for I2c {
    async fn read(&mut self, address: SevenBitAddress, read: &mut [u8]) -> Result<(), Self::Error> {
        self.twim.read(address, read).await
    }

    async fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), Self::Error> {
        self.twim.write(address, write).await
    }

    async fn write_read(
        &mut self,
        address: SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.twim.write_read(address, write, read).await
    }

    /// # Panics
    ///
    /// This panics with a `todo!` as [`embassy_nrf`] does *not* support transactions
    /// https://github.com/embassy-rs/embassy/blob/4d4cbc0dd3e84dfd7d29d1ecdd2b388568be081f/embassy-nrf/src/twim.rs#L875
    async fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.twim.transaction(address, operations).await
    }
}

impl embedded_hal_async::i2c::ErrorType for I2c {
    type Error = embassy_nrf::twim::Error;
}
