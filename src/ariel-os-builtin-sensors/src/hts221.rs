use ariel_os_debug::log::{debug, error};
use ariel_os_embassy::{api::time::Timer, asynch::Spawner, i2c::controller::I2cDevice};
use ariel_os_sensors::{
    Category, Label, MeasurementUnit, Sensor,
    sensor::{
        Accuracy, Mode as SensorMode, ReadingChannel, ReadingChannels, ReadingError, ReadingWaiter,
        Sample, Samples, SetModeError, State, TriggerMeasurementError,
    },
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};
use embedded_hal_async::i2c::I2c;
use hts221_async::Hts221;

use crate::{sensor_signaling::SensorSignaling, state_atomic::StateAtomic};

const PART_NUMBER: &str = "HTS221";

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

ariel_os_embassy::hal::define_peripherals!(Peripherals {});

pub struct Hts221I2c {
    state: StateAtomic,
    label: Option<&'static str>,
    sensor: OnceLock<Mutex<CriticalSectionRawMutex, Hts221<I2cDevice>>>,
    // config: Config,
    signaling: SensorSignaling,
}

impl Hts221I2c {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            sensor: OnceLock::new(),
            // config: Config {},
            signaling: SensorSignaling::new(),
        }
    }

    pub async fn init(
        &'static self,
        _spawner: Spawner,
        peripherals: Peripherals,
        i2c_device: I2cDevice,
        config: Config,
    ) {
        if !self.sensor.is_set() {
            // FIXME
            // self.config = config;

            let mut hts221 = Hts221::new(i2c_device);
            // FIXME: remove the unwrap
            if hts221.initialize().await.is_err() {
                error!("driver for HTS221 sensor failed to initialize");
            }

            let _ = self.sensor.init(Mutex::new(hts221));

            self.state.set(State::Enabled);
            debug!("{} enabled", PART_NUMBER);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            // FIXME: remove the unwrap and return an error instead
            let temp = match self.sensor.get().await.lock().await.read().await {
                Ok(acquisition) => acquisition.temperature.raw_value(),
                Err(err) => {
                    // FIXME
                    continue;
                }
            };

            let temp = (temp * 10.) as i32;

            let accuracy = accuracy(temp);
            let sample = Sample::new(temp, accuracy);

            self.signaling.signal_reading(Samples::V1([sample])).await;
        }
    }
}

impl Sensor for Hts221I2c {
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
        &[Category::HumidityTemperature]
    }

    fn reading_channels(&self) -> ReadingChannels {
        ReadingChannels::V1([
            // FIXME
            // ReadingChannel::new(
            //     Label::Humidity,
            //     // FIXME
            //     MeasurementUnit::PercentageRelativeHumidity,
            // ),
            ReadingChannel::new(
                Label::Temperature,
                -1, // FIXME: check this
                MeasurementUnit::Celsius,
            ),
        ])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("relative humidity & temperature")
    }

    fn part_number(&self) -> Option<&'static str> {
        Some(PART_NUMBER)
    }

    fn version(&self) -> u8 {
        0
    }
}

fn accuracy(temp: i32) -> Accuracy {
    // Table 3 of the datasheet.
    // Accuracy of 0.5 °C between 15 °C and 40 °C
    if -500 < temp && temp < 5500 {
        return Accuracy::SymmetricalError {
            deviation: 5,
            bias: 0,
            scaling: -1,
        };
    }

    // Accuracy of 1.0 °C otherwise
    return Accuracy::SymmetricalError {
        deviation: 10,
        bias: 0,
        scaling: -1,
    };
}
