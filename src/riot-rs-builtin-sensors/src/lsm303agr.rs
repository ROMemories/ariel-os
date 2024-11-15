use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex,
    once_lock::OnceLock, signal::Signal,
};
use embassy_time::{Duration, Timer};
use embedded_hal_async::delay::DelayNs;
use lsm303agr::{
    interface::I2cInterface, mode::MagOneShot, AccelMode, AccelOutputDataRate, AccelScale,
    Lsm303agr, MagMode, MagOutputDataRate,
};
use riot_rs_embassy::{arch, gpio, i2c::controller::I2cDevice, Spawner};
use riot_rs_sensors::{
    interrupts::{DeviceInterrupt, InterruptError, InterruptEvent, InterruptPin},
    sensor::{
        Accuracy, Mode as SensorMode, ReadingAxes, ReadingAxis, ReadingError, ReadingResult,
        ReadingWaiter, SetModeError, State, TriggerMeasurementError, Value, Values,
    },
    sensor_signaling::SensorSignaling,
    state_atomic::StateAtomic,
    Category, Label, MeasurementUnit, Sensor,
};

// FIXME: allow to disable the accel or the mag
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
    int1_accel: OnceLock<Mutex<CriticalSectionRawMutex, gpio::IntEnabledInput>>,
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
            int1_accel: OnceLock::new(),
        }
    }

    pub async fn init(
        &self,
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

            let mut accel_turn_on_delay = embassy_time::Delay;
            // FIXME: configuration
            driver
                .set_accel_mode_and_odr(
                    &mut accel_turn_on_delay,
                    AccelMode::Normal,
                    AccelOutputDataRate::Hz100,
                )
                .await
                .unwrap();
            // driver.set_accel_scale(AccelScale::G2).await.unwrap();
            // let mut mag_turn_on_delay = embassy_time::Delay;
            // // FIXME: configuration
            // driver
            //     .set_mag_mode_and_odr(
            //         &mut mag_turn_on_delay,
            //         MagMode::HighResolution,
            //         MagOutputDataRate::Hz10,
            //     )
            //     .await
            //     .unwrap();
            // let Ok(driver) = driver.into_mag_continuous().await else {
            //     // FIXME
            //     panic!();
            // };

            let _ = self.sensor.init(Mutex::new(driver));

            self.state.set(State::Enabled);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            let (accel_data, mag_data) = {
                let mut sensor = self.sensor.get().await.lock().await;

                loop {
                    // FIXME: remove this unwrap
                    if sensor.accel_status().await.unwrap().xyz_new_data() {
                        break;
                    }
                    // FIXME: adjust this delay
                    Timer::after(Duration::from_millis(2)).await;
                }

                // `magnetic_field()` already checks `mag_status()` internally
                match (sensor.acceleration().await, sensor.magnetic_field().await) {
                    (Ok(accel_data), Ok(mag_data)) => (accel_data, mag_data),
                    (Err(lsm303agr::Error::Comm(err)), Ok(_))
                    | (Ok(_), Err(lsm303agr::Error::Comm(err))) => {
                        self.signaling
                            .signal_reading_err(ReadingError::SensorAccess)
                            .await;
                        continue;
                    }
                    _ => unreachable!(), // FIXME: is it?
                }
            };

            // FIXME
            let accuracy = Accuracy::Unknown;

            let accel_x = Value::new(accel_data.x_mg(), accuracy);
            let accel_y = Value::new(accel_data.y_mg(), accuracy);
            let accel_z = Value::new(accel_data.z_mg(), accuracy);
            let mag_x = Value::new(mag_data.x_nt(), accuracy);
            let mag_y = Value::new(mag_data.y_nt(), accuracy);
            let mag_z = Value::new(mag_data.z_nt(), accuracy);

            self.signaling
                .signal_reading(Values::V6([accel_x, accel_y, accel_z, mag_x, mag_y, mag_z]))
                .await;
        }
    }

    // TODO: maybe make this async in case we need to access a mutex? may be annoying to use during
    // non-async init
    pub fn register_interrupt_pin(
        &self,
        pin: gpio::IntEnabledInput,
        device_interrupt: DeviceInterrupt,
    ) {
        let _ = self.int1_accel.init(Mutex::new(pin));
        // FIXME: store/do something with DeviceInterrupt, InterruptPin
    }

    // TODO: maybe make this async in case we need to access a mutex? may be annoying to use during
    // non-async init
    pub fn use_interrupt_for_data_ready_event(&self) {
        todo!();
    }

    pub async fn wait_for_interrupt_event(
        &self,
        event: InterruptEvent,
    ) -> Result<(), InterruptError> {
        self.sensor
            .get()
            .await
            .lock()
            .await
            .acc_set_int1_threshold(lsm303agr::AccelerationThreshold1::try_new(0x0c).unwrap())
            .await
            .unwrap();

        self.sensor
            .get()
            .await
            .lock()
            .await
            .acc_set_int1_duration(lsm303agr::AccelerationDuration::try_new(5).unwrap())
            .await
            .unwrap();

        self.sensor
            .get()
            .await
            .lock()
            .await
            .acc_enable_interrupt(lsm303agr::Interrupt::Aoi1)
            .await
            .unwrap();

        loop {
            self.int1_accel
                .get()
                .await
                .lock()
                .await
                .wait_for_low()
                .await;

            // Check the interrupt bit on the sensor device
            if self
                .sensor
                .get()
                .await
                .lock()
                .await
                .acc_reset_int1()
                .await
                .unwrap()
            {
                break;
            }
        }

        Ok(())
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
        ReadingAxes::V6([
            ReadingAxis::new(Label::AccelX, -3, MeasurementUnit::AccelG),
            ReadingAxis::new(Label::AccelY, -3, MeasurementUnit::AccelG),
            ReadingAxis::new(Label::AccelZ, -3, MeasurementUnit::AccelG),
            ReadingAxis::new(Label::MagX, -9, MeasurementUnit::Tesla), // FIXME: fix the scaler
            ReadingAxis::new(Label::MagY, -9, MeasurementUnit::Tesla),
            ReadingAxis::new(Label::MagZ, -9, MeasurementUnit::Tesla),
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
