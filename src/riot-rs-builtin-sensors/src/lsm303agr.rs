use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex,
    once_lock::OnceLock, signal::Signal,
};
use embedded_hal_async::delay::DelayNs;
use lsm303agr::{
    interface::I2cInterface, mode::MagOneShot, AccelMode, AccelOutputDataRate, Lsm303agr,
};
use riot_rs_embassy::{arch, gpio, i2c::controller::I2cDevice, Spawner};
use riot_rs_sensors::{
    sensor::{
        Accuracy, Mode as SensorMode, ReadingAxes, ReadingAxis, ReadingError, ReadingResult,
        ReadingWaiter, SetModeError, State, TriggerMeasurementError, Value, Values,
    },
    sensor_signaling::SensorSignaling,
    state_atomic::StateAtomic,
    Category,
    Label,
    MeasurementUnit,
    Sensor,
};

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
    // FIXME: find a way to change the mag mode
    sensor:
        OnceLock<Mutex<CriticalSectionRawMutex, Lsm303agr<I2cInterface<I2cDevice>, MagOneShot>>>,
    signaling: SensorSignaling,
}

impl Lsm303agrI2c {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            sensor: OnceLock::new(),
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

            let _ = self.sensor.init(Mutex::new(driver));

            self.state.set(State::Enabled);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            // FIXME: wait for data to be ready using `accel_status()`

            let data = match self.sensor.get().await.lock().await.acceleration().await {
                Ok(data) => data,
                Err(lsm303agr::Error::Comm(err)) => {
                    self.signaling
                        .signal_reading_err(ReadingError::SensorAccess)
                        .await;
                    continue;
                }
                Err(_) => unreachable!(), // FIXME: is it?
            };

            // FIXME
            let accuracy = Accuracy::Unknown;

            let x = Value::new(data.x_mg(), accuracy);
            let y = Value::new(data.y_mg(), accuracy);
            let z = Value::new(data.z_mg(), accuracy);

            self.signaling.signal_reading(Values::V3([x, y, z])).await;
        }
    }
}

impl Sensor for Lsm303agrI2c {
    fn trigger_measurement(&self) -> Result<(), TriggerMeasurementError> {
        if self.state.get() != State::Enabled {
            return Err(TriggerMeasurementError::NonEnabled);
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

    fn set_mode(&self, mode: SensorMode) -> Result<State, SetModeError> {
        let new_state = self.state.set_mode(mode);

        if new_state == State::Uninitialized {
            Err(SetModeError::Uninitialized)
        } else {
            Ok(new_state)
        }
    }

    fn state(&self) -> State {
        self.state.get()
    }

    fn categories(&self) -> &'static [Category] {
        // FIXME: add AccelerometerMagnetometer
        &[Category::Accelerometer]
    }

    fn reading_axes(&self) -> ReadingAxes {
        // FIXME: add magnetometer readings
        ReadingAxes::V3([
            ReadingAxis::new(Label::X, -3, MeasurementUnit::AccelG),
            ReadingAxis::new(Label::Y, -3, MeasurementUnit::AccelG),
            ReadingAxis::new(Label::Z, -3, MeasurementUnit::AccelG),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_type_sizes() {
        assert_eq!(size_of::<Values>(), 6 * size_of::<u32>());
        assert_eq!(size_of::<Lsm303agrI2c>(), 46 * size_of::<u32>());
    }
}
