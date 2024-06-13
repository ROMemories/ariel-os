use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_futures::select::Either;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_aht20::Aht20 as InnerAht20;
use portable_atomic::{AtomicU8, Ordering};
use riot_rs_embassy::Spawner;
use riot_rs_sensors::{
    sensor::{
        AccuracyError, Mode, PhysicalValue, PhysicalValues, ReadingError, ReadingInfo,
        ReadingInfos, ReadingResult, State,
    },
    Category, Label, PhysicalUnit, Sensor,
};

// FIXME: what's the best way to instantiate sensor driver configuration?
#[derive(Debug)]
#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

riot_rs_embassy::define_peripherals!(Peripherals {});

pub type Aht20I2c = Aht20<riot_rs_embassy::arch::i2c::I2c>;

pub struct Aht20<I2C: embedded_hal_async::i2c::I2c + 'static> {
    state: AtomicU8,
    label: Option<&'static str>,
    // TODO: consider using MaybeUninit?
    ht: Mutex<
        CriticalSectionRawMutex,
        Option<InnerAht20<I2cDevice<'static, CriticalSectionRawMutex, I2C>, Delay>>,
    >,
}

impl<I2C: embedded_hal_async::i2c::I2c> Aht20<I2C> {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: AtomicU8::new(State::Uninitialized as u8),
            label,
            ht: Mutex::new(None),
        }
    }

    pub fn init(
        &'static self,
        _spawner: Spawner,
        peripherals: Peripherals,
        i2c: I2cDevice<'static, CriticalSectionRawMutex, I2C>,
        config: Config,
    ) {
        if self.state.load(Ordering::Acquire) == State::Uninitialized as u8 {
            let mut delay = Delay {}; // FIXME: set a delay? what does it even do?
                                      // TODO: it seems there is no alternate address, still allow users to set it?
                                      // FIXME: handle the error
            let driver = embassy_futures::block_on(InnerAht20::new(i2c, 0x38, delay)).unwrap();

            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut ht = self.ht.try_lock().unwrap();
            *ht = Some(driver);

            self.state.store(State::Enabled as u8, Ordering::Release);
        }
    }
}

impl<I2C: embedded_hal_async::i2c::I2c + Send> Sensor for Aht20<I2C> {
    #[allow(refining_impl_trait)]
    async fn measure(&self) -> ReadingResult<PhysicalValues> {
        if self.state.load(Ordering::Acquire) == State::Enabled as u8 {
            return Err(ReadingError::NonEnabled);
        }

        // TODO: maybe should check is_data_ready()?
        let data = self
            .ht
            .lock()
            .await
            .as_mut()
            .unwrap()
            .measure()
            .await
            .map_err(|_| ReadingError::SensorAccess)?;

        #[allow(clippy::cast_possible_truncation)]
        // FIXME: dumb scaling, take precision into account
        // FIXME: specify the measurement error
        Ok(PhysicalValues::V2([
            PhysicalValue::new(
                (data.relative_humidity * 100.) as i32,
                AccuracyError::Unknown,
            ),
            // Celsius typo in the library
            PhysicalValue::new(
                (data.temperature.celcius() * 100.) as i32,
                AccuracyError::Unknown,
            ),
        ]))
    }

    fn set_mode(&self, mode: Mode) {
        if self.state.load(Ordering::Acquire) != State::Uninitialized as u8 {
            let state = State::from(mode);
            self.state.store(state as u8, Ordering::Release);
        }
        // TODO: return an error otherwise?
    }

    fn state(&self) -> State {
        let state = self.state.load(Ordering::Acquire);
        // NOTE(no-panic): the state atomic is only written from a State
        State::try_from(state).unwrap()
    }

    fn categories(&self) -> &'static [Category] {
        &[
            Category::HumidityTemperature,
            Category::Humidity,
            Category::Temperature,
        ]
    }

    fn reading_infos(&self) -> ReadingInfos {
        ReadingInfos::V2([
            ReadingInfo::new(Label::Humidity, -2, PhysicalUnit::Percent),
            ReadingInfo::new(Label::Temperature, -2, PhysicalUnit::Celsius),
        ])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("Humidity & temperature sensor") // FIXME: ?
    }

    fn part_number(&self) -> &'static str {
        "AHT20"
    }

    fn version(&self) -> u8 {
        0
    }
}

// FIXME: introduce and implement a sensor category
