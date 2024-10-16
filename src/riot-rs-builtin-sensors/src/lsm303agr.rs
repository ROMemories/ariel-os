use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex, signal::Signal,
};
use embedded_hal_async::delay::DelayNs;
use lsm303agr::{
    interface::I2cInterface, mode::MagOneShot, AccelMode, AccelOutputDataRate, Lsm303agr,
};
use riot_rs_embassy::{arch, gpio, i2c::controller::I2cDevice, Spawner};
use riot_rs_sensors::{
    interrupts::{
        self, AccelerometerInterruptEvent, DeviceInterrupt, InterruptError, InterruptEvent,
        InterruptEventKind,
    },
    sensor::{
        AccuracyError, MeasurementError, Mode as SensorMode, ModeSettingError, PhysicalValue,
        PhysicalValues, ReadingAxes, ReadingAxis, ReadingError, ReadingResult, ReadingWaiter,
        SensorSignaling, State, StateAtomic,
    },
    Category, Label, PhysicalUnit, Sensor,
};

// TODO: add support for SPI

#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    // FIXME
}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

riot_rs_embassy::define_peripherals!(Peripherals {});

pub struct Lsm303agrI2c {
    state: StateAtomic,
    label: Option<&'static str>,
    // TODO: consider using MaybeUninit?
    // FIXME: find a way to change the mag mode
    sensor: Mutex<CriticalSectionRawMutex, Option<Lsm303agr<I2cInterface<I2cDevice>, MagOneShot>>>,
    signaling: SensorSignaling,
}

impl Lsm303agrI2c {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            sensor: Mutex::new(None),
            signaling: SensorSignaling::new(),
        }
    }

    pub async fn init(
        &'static self,
        _spawner: Spawner,
        peripherals: Peripherals,
        i2c: I2cDevice,
        config: Config,
    ) {
        if self.state.get() == State::Uninitialized {
            let mut driver = Lsm303agr::new_with_i2c(i2c);

            if driver.init().await.is_err() {
                return;
            }

            // FIXME: configuration
            let mut turn_on_delay = embassy_time::Delay;
            driver
                .set_accel_mode_and_odr(
                    &mut turn_on_delay,
                    AccelMode::Normal,
                    AccelOutputDataRate::Hz10,
                )
                .await
                .unwrap();

            let mut sensor = self.sensor.try_lock().unwrap();
            *sensor = Some(driver);

            self.state.set(State::Enabled);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            // FIXME: wait for data to be ready using `accel_status()`

            let data = match self
                .sensor
                .lock()
                .await
                .as_mut()
                .unwrap()
                .acceleration()
                .await
            {
                Ok(data) => data,
                Err(lsm303agr::Error::Comm(err)) => {
                    self.signaling
                        .signal_reading_err(ReadingError::SensorAccess)
                        .await;
                    continue;
                }
                Err(_) => unreachable!(), // FIXME: is it?
            };

            let x = PhysicalValue::new(data.x_mg());
            let y = PhysicalValue::new(data.y_mg());
            let z = PhysicalValue::new(data.z_mg());

            self.signaling
                .signal_reading(PhysicalValues::V3([x, y, z]))
                .await;
        }
    }
}

impl Sensor for Lsm303agrI2c {
    fn trigger_measurement(&self) -> Result<(), MeasurementError> {
        if self.state.get() != State::Enabled {
            return Err(MeasurementError::NonEnabled);
        }

        self.signaling.trigger_measurement();

        Ok(())
    }

    fn wait_for_reading(&'static self) -> ReadingWaiter {
        if self.state.get() != State::Enabled {
            return ReadingWaiter::Err(ReadingError::NonEnabled);
        }

        self.signaling.wait_for_reading()
    }

    fn available_interrupt_events(&self) -> &[InterruptEventKind] {
        // FIXME
        &[]
    }

    fn set_mode(&self, mode: SensorMode) -> Result<State, ModeSettingError> {
        let new_state = self.state.set_mode(mode);

        if new_state == State::Uninitialized {
            Err(ModeSettingError::Uninitialized)
        } else {
            Ok(new_state)
        }
    }

    fn state(&self) -> State {
        self.state.get()
    }

    fn categories(&self) -> &'static [Category] {
        // FIXME: add AccelerometerMagnetometer
        // FIXME: add Temperature (should we have a AccelerometerMagnetometerTemperature category?)
        &[Category::Accelerometer]
    }

    fn reading_axes(&self) -> ReadingAxes {
        fn accuracy(value: PhysicalValue) -> AccuracyError {
            AccuracyError::Symmetrical {
                deviation: 40,
                bias: -30,
                scaling: -3,
            }
        }

        // FIXME: add magnetometer readings
        // FIXME: also return the temperature? (needs a new Cargo feature for V7)
        ReadingAxes::V3([
            ReadingAxis::new(Label::X, -3, PhysicalUnit::AccelG, accuracy),
            ReadingAxis::new(Label::Y, -3, PhysicalUnit::AccelG, accuracy),
            ReadingAxis::new(Label::Z, -3, PhysicalUnit::AccelG, accuracy),
        ])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("3-axis accelerometer & magnetometer")
    }

    fn part_number(&self) -> Option<&'static str> {
        Some("LSM303AGR")
    }

    fn version(&self) -> u8 {
        0
    }
}
