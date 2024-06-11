use embassy_futures::select::Either;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use lis3dh_async::{Configuration, DataRate, Lis3dh as InnerLis3dh, Lis3dhSPI};
use portable_atomic::{AtomicBool, AtomicI32, Ordering};
use riot_rs_embassy::{arch, Spawner};
use riot_rs_sensors::{
    sensor::{
        MeasurementError, PhysicalValue, PhysicalValues, Reading, ReadingError, ReadingInfo,
        ReadingInfos, ReadingResult,
    },
    Category, Label, PhysicalUnit, Sensor,
};

use crate::lis3dh::Config;

pub use lis3dh_async::{Mode, SlaveAddr as Address};

// TODO: could maybe use a OnceCell instead of an Option
pub struct Lis3dhSpi {
    initialized: AtomicBool, // TODO: use an atomic bitset for initialized and enabled
    enabled: AtomicBool,
    label: Option<&'static str>,
    // TODO: consider using MaybeUninit?
    accel: Mutex<CriticalSectionRawMutex, Option<InnerLis3dh<Lis3dhSPI<arch::spi::SpiDevice>>>>,
}

impl Lis3dhSpi {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
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
        // peripherals: Peripherals,
        spi: arch::spi::SpiDevice,
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
            //     InnerLis3dh::new_spi_with_config(spi, config),
            //     Timer::after(Duration::from_secs(1)),
            // ));
            // let driver = match init {
            //     Either::First(driver) => driver.unwrap(),
            //     Either::Second(_) => panic!("timeout when initializing Lis3dh"), // FIXME
            // };

            let driver =
                embassy_futures::block_on(InnerLis3dh::new_spi_with_config(spi, lis3dh_config))
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

impl Sensor for Lis3dhSpi {
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

    fn categories(&self) -> &'static [Category] {
        &[Category::Accelerometer]
    }

    fn reading_infos(&self) -> ReadingInfos {
        ReadingInfos::V3([
            ReadingInfo::new(Label::X, -2, PhysicalUnit::AccelG),
            ReadingInfo::new(Label::Y, -2, PhysicalUnit::AccelG),
            ReadingInfo::new(Label::Z, -2, PhysicalUnit::AccelG),
        ])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
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
