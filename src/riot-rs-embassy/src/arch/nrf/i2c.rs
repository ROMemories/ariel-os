use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice as InnerI2cDevice;
use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    twim::{InterruptHandler, Twim},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_hal_async::i2c::{Operation, SevenBitAddress};

pub use embassy_nrf::twim::Frequency;

// TODO: factor this out (across archs)?
// TODO: do we need a CriticalSectionRawMutex here?
pub type I2cDevice = InnerI2cDevice<'static, CriticalSectionRawMutex, I2c>;

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

// FIXME
bind_interrupts!(
    struct Irqs {
        SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => InterruptHandler<peripherals::TWISPI0>;
        SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => InterruptHandler<peripherals::TWISPI1>;
    }
);

macro_rules! define_i2c_driver {
    ($peripheral:ident) => {
        paste::paste! {
            pub struct [<I2c $peripheral>] {
                twim: Twim<'static, peripherals::$peripheral>,
            }

            impl [<I2c $peripheral>] {
                #[must_use]
                pub fn new(
                    twim_peripheral: peripherals::$peripheral,
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

            impl embedded_hal_async::i2c::I2c for [<I2c $peripheral>] {
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

            impl embedded_hal_async::i2c::ErrorType for [<I2c $peripheral>] {
                type Error = embassy_nrf::twim::Error;
            }
        }
    }
}

// FIXME: support other nRF archs
define_i2c_driver!(TWISPI0);
define_i2c_driver!(TWISPI1);

// TODO: codegen this
pub enum I2c {
    TWISPI0(I2cTWISPI0),
    TWISPI1(I2cTWISPI1),
}

impl embedded_hal_async::i2c::I2c for I2c {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        match self {
            Self::TWISPI0(i2c) => i2c.twim.read(address, read).await,
            Self::TWISPI1(i2c) => i2c.twim.read(address, read).await,
        }
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        match self {
            Self::TWISPI0(i2c) => i2c.twim.write(address, write).await,
            Self::TWISPI1(i2c) => i2c.twim.write(address, write).await,
        }
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        match self {
            Self::TWISPI0(i2c) => i2c.twim.write_read(address, write, read).await,
            Self::TWISPI1(i2c) => i2c.twim.write_read(address, write, read).await,
        }
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        match self {
            Self::TWISPI0(i2c) => i2c.twim.transaction(address, operations).await,
            Self::TWISPI1(i2c) => i2c.twim.transaction(address, operations).await,
        }
    }
}

impl embedded_hal_async::i2c::ErrorType for I2c {
    type Error = embassy_nrf::twim::Error;
}
