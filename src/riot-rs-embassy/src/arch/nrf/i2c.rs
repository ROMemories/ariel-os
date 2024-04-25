use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    twim::{InterruptHandler, Twim},
    Peripheral,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_hal_async::i2c::{Operation, SevenBitAddress};

// FIXME: maybe we should provide our own type, unified across archs
pub use embassy_nrf::twim::{Config, Frequency};

// FIXME: does this prevent us from binding another interrupt handler to the same interrupt,
// elsewhere?
bind_interrupts!(
    struct Irqs {
        SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => InterruptHandler<peripherals::TWISPI0>;
    }
);

// FIXME: support other I2C peripherals as well
pub struct I2c {
    twim: Mutex<CriticalSectionRawMutex, Twim<'static, peripherals::TWISPI0>>,
}

impl I2c {
    #[must_use]
    pub fn new(
        twim_peripheral: peripherals::TWISPI0,
        sda_pin: impl Peripheral<P = impl GpioPin> + 'static,
        scl_pin: impl Peripheral<P = impl GpioPin> + 'static,
        config: Config,
    ) -> Self {
        let twim = Twim::new(twim_peripheral, Irqs, sda_pin, scl_pin, config);

        Self {
            twim: Mutex::new(twim),
        }
    }
}

impl embedded_hal_async::i2c::I2c for I2c {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.twim.lock().await.read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.twim.lock().await.write(address, write).await
    }

    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.twim.lock().await.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: SevenBitAddress, // FIXME: support 10-bit addressing as well
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        // embassy_nrf does *not* support transactions
        // https://github.com/embassy-rs/embassy/blob/4d4cbc0dd3e84dfd7d29d1ecdd2b388568be081f/embassy-nrf/src/twim.rs#L875
        self.twim
            .lock()
            .await
            .transaction(address, operations)
            .await
    }
}

impl embedded_hal_async::i2c::ErrorType for I2c {
    type Error = embassy_nrf::twim::Error;
}
