use embassy_rp::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    i2c::{InterruptHandler, SclPin, SdaPin},
    peripherals, Peripheral,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_hal_async::i2c::{Operation, SevenBitAddress};

// FIXME: maybe we should provide our own config type, unified across archs
pub use embassy_rp::i2c::Config;

bind_interrupts!(
    struct Irqs {
        I2C0_IRQ => InterruptHandler<peripherals::I2C0>;
    }
);

// FIXME: support other I2C peripherals as well
pub struct I2c {
    i2c: Mutex<
        CriticalSectionRawMutex,
        embassy_rp::i2c::I2c<'static, peripherals::I2C0, embassy_rp::i2c::Async>,
    >,
}

impl I2c {
    #[must_use]
    pub fn new(
        i2c_peripheral: peripherals::I2C0,
        sda_pin: impl SdaPin<peripherals::I2C0>,
        scl_pin: impl SclPin<peripherals::I2C0>,
        config: Config,
    ) -> Self {
        let i2c = embassy_rp::i2c::I2c::new_async(i2c_peripheral, scl_pin, sda_pin, Irqs, config);

        Self {
            i2c: Mutex::new(i2c),
        }
    }
}

impl embedded_hal_async::i2c::I2c for I2c {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.lock().await.read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.i2c.lock().await.write(address, write).await
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c.lock().await.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.i2c.lock().await.transaction(address, operations).await
    }
}

impl embedded_hal_async::i2c::ErrorType for I2c {
    type Error = embassy_rp::i2c::Error;
}
