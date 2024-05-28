use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_futures::select::Either;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use lis3dh_async::{Configuration, DataRate, Lis3dh as InnerLis3dh, Lis3dhI2C};
use portable_atomic::{AtomicBool, Ordering};
use riot_rs_embassy::Spawner;
use riot_rs_sensors::{
    label::Label,
    sensor::{
        Category, Labels, MeasurementError, NotificationReceiver, PhysicalUnits, PhysicalValue,
        PhysicalValues, ReadingError, ReadingResult, ThresholdKind, ValueScales,
    },
    PhysicalUnit, Sensor,
};

pub use crate::lis3dh_spi::Lis3dhSpi;
pub use lis3dh_async::{Mode, SlaveAddr as Address};

// FIXME: what's the best way to instantiate sensor driver configuration?
#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    pub address: Address,
    pub mode: Mode,
    pub datarate: DataRate,
    pub enable_x_axis: bool,
    pub enable_y_axis: bool,
    pub enable_z_axis: bool,
    pub block_data_update: bool, // TODO: do we need to expose this?
    pub enable_temperature: bool,
}

impl Default for Config {
    fn default() -> Self {
        let config = Configuration::default();
        Self {
            address: Address::Alternate,
            mode: config.mode,
            datarate: config.datarate,
            enable_x_axis: config.enable_x_axis,
            enable_y_axis: config.enable_y_axis,
            enable_z_axis: config.enable_z_axis,
            block_data_update: config.block_data_update,
            enable_temperature: config.enable_temperature,
        }
    }
}

riot_rs_embassy::define_peripherals!(Peripherals {});

pub type Lis3dhI2c = Lis3dh<riot_rs_embassy::arch::i2c::I2c>;

// TODO: support SPI as well
// TODO: could maybe use a OncelCell instead of an Option
pub struct Lis3dh<I2C: embedded_hal_async::i2c::I2c + 'static> {
    initialized: AtomicBool, // TODO: use an atomic bitset for initialized and enabled
    enabled: AtomicBool,
    label: &'static str,
    // TODO: consider using MaybeUninit?
    accel: Mutex<
        CriticalSectionRawMutex,
        Option<InnerLis3dh<Lis3dhI2C<I2cDevice<'static, CriticalSectionRawMutex, I2C>>>>,
    >,
}

// TODO: need to impl Lis3dhCore?

impl<I2C: embedded_hal_async::i2c::I2c> Lis3dh<I2C> {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: &'static str) -> Self {
        Self {
            initialized: AtomicBool::new(false),
            enabled: AtomicBool::new(false),
            label,
            accel: Mutex::new(None),
        }
    }

    pub fn init(
        &'static self,
        _spawner: Spawner,
        peripherals: Peripherals,
        i2c: I2cDevice<'static, CriticalSectionRawMutex, I2C>,
        config: Config,
    ) {
        if !self.initialized.load(Ordering::Acquire) {
            // TODO: can this be made shorter?
            let mut lis3dh_config = Configuration::default();
            lis3dh_config.mode = config.mode;
            lis3dh_config.datarate = config.datarate;
            lis3dh_config.enable_x_axis = config.enable_x_axis;
            lis3dh_config.enable_y_axis = config.enable_y_axis;
            lis3dh_config.enable_z_axis = config.enable_z_axis;
            lis3dh_config.block_data_update = config.block_data_update;
            lis3dh_config.enable_temperature = config.enable_temperature;

            // FIXME: add a timeout, blocks indefinitely if no device is connected
            // FIXME: is using block_on ok here?
            // FIXME: handle the Result
            // FIXME: this does not work because of https://github.com/embassy-rs/embassy/issues/2830
            // let init = embassy_futures::block_on(embassy_futures::select::select(
            //     InnerLis3dh::new_i2c_with_config(i2c, addr, config),
            //     Timer::after(Duration::from_secs(1)),
            // ));
            // let driver = match init {
            //     Either::First(driver) => driver.unwrap(),
            //     Either::Second(_) => panic!("timeout when initializing Lis3dh"), // FIXME
            // };

            let driver = embassy_futures::block_on(InnerLis3dh::new_i2c_with_config(
                i2c,
                config.address,
                lis3dh_config,
            ))
            .unwrap();

            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut accel = self.accel.try_lock().unwrap();
            *accel = Some(driver);

            self.initialized.store(true, Ordering::Release);
            self.enabled.store(true, Ordering::Release);
        }
    }
}

impl<I2C: embedded_hal_async::i2c::I2c + Send> Sensor for Lis3dh<I2C> {
    async fn read(&self) -> ReadingResult<PhysicalValues> {
        if !self.enabled.load(Ordering::Acquire) {
            return Err(ReadingError::Disabled);
        }

        // TODO: maybe should check is_data_ready()?
        let data = self
            .accel
            .lock()
            .await
            .as_mut()
            .unwrap()
            .accel_norm()
            .await
            .map_err(|_| ReadingError::SensorAccess)?;

        #[allow(clippy::cast_possible_truncation)]
        // FIXME: dumb scaling, take precision into account
        // FIXME: specify the measurement error
        let x = PhysicalValue::new((data.x * 100.) as i32, MeasurementError::Unknown);
        let y = PhysicalValue::new((data.y * 100.) as i32, MeasurementError::Unknown);
        let z = PhysicalValue::new((data.z * 100.) as i32, MeasurementError::Unknown);

        Ok(PhysicalValues::V3([x, y, z]))
    }

    fn set_enabled(&self, enabled: bool) {
        if self.initialized.load(Ordering::Acquire) {
            self.enabled.store(enabled, Ordering::Release);
        }
        // TODO: return an error otherwise?
    }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    fn set_threshold(&self, kind: ThresholdKind, value: PhysicalValue) {
        todo!()
    }

    fn set_threshold_enabled(&self, kind: ThresholdKind, enabled: bool) {
        todo!()
    }

    fn subscribe(&self) -> NotificationReceiver {
        todo!()
    }

    fn category(&self) -> Category {
        Category::Accelerometer
    }

    fn value_scales(&self) -> ValueScales {
        ValueScales::V3([-2, -2, -2])
    }

    fn units(&self) -> PhysicalUnits {
        // FIXME: what's the actual unit?
        PhysicalUnits::V3([
            PhysicalUnit::AccelG,
            PhysicalUnit::AccelG,
            PhysicalUnit::AccelG,
        ])
    }

    fn reading_labels(&self) -> Labels {
        Labels::V3([Label::X, Label::Y, Label::Z])
    }

    fn label(&self) -> &'static str {
        &self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("3-axis accelerometer")
    }

    fn part_number(&self) -> &'static str {
        "LIS3DH"
    }

    fn version(&self) -> u8 {
        0
    }
}

// TODO: consider accelerometer.rs as well
// impl Accelerometer for Lis3dh {
//
// }
