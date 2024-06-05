use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice as InnerI2cDevice;
use embassy_rp::{
    bind_interrupts,
    i2c::{InterruptHandler, SclPin, SdaPin},
    peripherals,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_hal_async::i2c::Operation;

// TODO: factor this out (across archs)?
// TODO: do we need a CriticalSectionRawMutex here?
pub type I2cDevice = InnerI2cDevice<'static, CriticalSectionRawMutex, I2c>;

// We do not provide configuration for internal pull-ups as the RP2040 datasheet mentions in
// section 4.3.1.3 that the GPIO used should have pull-ups enabled.
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
        }
    }
}

// Possible values are copied from embassy-nrf
// TODO: check how well this matches the RP2040 capabilities
#[repr(u32)]
pub enum Frequency {
    K100 = 100_000,
    K250 = 250_000,
    K400 = 400_000,
}

// FIXME
bind_interrupts!(
    struct Irqs {
        I2C0_IRQ => InterruptHandler<peripherals::I2C0>;
        I2C1_IRQ => InterruptHandler<peripherals::I2C1>;
    }
);

macro_rules! define_i2c_driver {
    ($peripheral:ident) => {
        paste::paste! {
            pub struct [<I2c $peripheral>] {
                i2c: embassy_rp::i2c::I2c<'static, peripherals::$peripheral, embassy_rp::i2c::Async>,
            }

            impl [<I2c $peripheral>] {
                #[must_use]
                pub fn new(
                    i2c_peripheral: peripherals::$peripheral,
                    sda_pin: impl SdaPin<peripherals::$peripheral>,
                    scl_pin: impl SclPin<peripherals::$peripheral>,
                    config: Config,
                ) -> Self {
                    let mut i2c_config = embassy_rp::i2c::Config::default();
                    i2c_config.frequency = config.frequency as u32;

                    let i2c =
                        embassy_rp::i2c::I2c::new_async(i2c_peripheral, scl_pin, sda_pin, Irqs, i2c_config);

                    Self {
                        i2c,
                    }
                }
            }

            impl embedded_hal_async::i2c::I2c for [<I2c $peripheral>] {
                async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
                    self.i2c.read(address, read).await
                }

                async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
                    self.i2c.write(address, write).await
                }

                async fn write_read(
                    &mut self,
                    address: u8,
                    write: &[u8],
                    read: &mut [u8],
                ) -> Result<(), Self::Error> {
                    self.i2c.write_read(address, write, read).await
                }

                async fn transaction(
                    &mut self,
                    address: u8,
                    operations: &mut [Operation<'_>],
                ) -> Result<(), Self::Error> {
                    self.i2c.transaction(address, operations).await
                }
            }

            impl embedded_hal_async::i2c::ErrorType for [<I2c $peripheral>] {
                type Error = embassy_rp::i2c::Error;
            }
        }
    }
}

define_i2c_driver!(I2C0);
define_i2c_driver!(I2C1);

// TODO: codegen this
pub enum I2c {
    I2C0(I2cI2C0),
    I2C1(I2cI2C1),
}

impl embedded_hal_async::i2c::I2c for I2c {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        match self {
            Self::I2C0(i2c) => i2c.i2c.read(address, read).await,
            Self::I2C1(i2c) => i2c.i2c.read(address, read).await,
        }
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        match self {
            Self::I2C0(i2c) => i2c.i2c.write(address, write).await,
            Self::I2C1(i2c) => i2c.i2c.write(address, write).await,
        }
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        match self {
            Self::I2C0(i2c) => i2c.i2c.write_read(address, write, read).await,
            Self::I2C1(i2c) => i2c.i2c.write_read(address, write, read).await,
        }
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        match self {
            Self::I2C0(i2c) => i2c.i2c.transaction(address, operations).await,
            Self::I2C1(i2c) => i2c.i2c.transaction(address, operations).await,
        }
    }
}

impl embedded_hal_async::i2c::ErrorType for I2c {
    type Error = embassy_rp::i2c::Error;
}
