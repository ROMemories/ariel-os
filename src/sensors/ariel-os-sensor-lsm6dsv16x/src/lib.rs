#![no_std]

use ariel_os_debug::log::debug;
use ariel_os_embassy::{api::time::Timer, asynch::Spawner};
use ariel_os_hal::i2c::controller::I2cDevice;
use ariel_os_sensors::{
    Category, Label, MeasurementUnit, Sensor,
    sensor::{
        Accuracy, Mode as SensorMode, ReadingChannel, ReadingChannels, ReadingError, ReadingWaiter,
        Sample, Samples, SetModeError, State, TriggerMeasurementError,
    },
};
use ariel_os_sensors_utils::{SensorSignaling, StateAtomic};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};
use embedded_hal_async::i2c::I2c;

const PART_NUMBER: &str = "LSM6DSV16X";

// FIXME: depends on the pin config
const TARGET_I2C_ADDR: u8 = 0b1101011 >> 1;

const WHO_AM_I_REG_ADDR: u8 = 0x0f;
const DEVICE_ID: u8 = 0x70;

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

pub struct Lsm6dsv16xI2c {
    state: StateAtomic,
    label: Option<&'static str>,
    i2c: OnceLock<Mutex<CriticalSectionRawMutex, I2cDevice>>,
    // config: Config,
    signaling: SensorSignaling,
}

impl Lsm6dsv16xI2c {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            i2c: OnceLock::new(),
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
        if !self.i2c.is_set() {
            // FIXME
            // self.config = config;

            let _ = self.i2c.init(Mutex::new(i2c_device));

            self.state.set(State::Enabled);
            debug!("{} enabled", PART_NUMBER);
        }
    }

    pub async fn run(&'static self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            // Sensor configuration
            // FIXME

            let mut i2c = self.i2c.get().await.lock().await;

            // Trigger one-shot measurement
            // FIXME

            if let Err(_err) = res {
                self.signaling
                    .signal_reading_err(ReadingError::SensorAccess)
                    .await;
                continue;
            }

            // FIXME
            // Wait for the measurement
            loop {
                let mut buf = [0u8];
                let res = i2c
                    .write_read(TARGET_I2C_ADDR, &[STATUS_REG_ADDR], &mut buf)
                    .await;

                // Not BUSY anymore
                if buf[0] & (1 << BUSY_BIT) == 0 {
                    break;
                }

                // TODO: configuration
                Timer::after_millis(10).await;
            }

            // FIXME
            // Reads both temperature bytes thanks to IF_ADD_INC.
            let mut buf = [0u8; 2];
            let res = i2c
                .write_read(TARGET_I2C_ADDR, &[TEMP_L_OUT_REG_ADDR], &mut buf)
                .await;

            // TODO: increases text size a bit; remove this?
            drop(i2c);

            if let Err(_err) = res {
                self.signaling
                    .signal_reading_err(ReadingError::SensorAccess)
                    .await;
                continue;
            }

            // FIXME
            // Smaller text size than using `i32::from_be_bytes()`
            let temp = i32::from(buf[1]) << 8 | i32::from(buf[0]);

            let accuracy = accuracy(temp);
            let sample = Sample::new(temp, accuracy);

            let mut samples = Samples::from([sample]);
            samples.set_driver_ref(self);
            self.signaling.signal_reading(samples).await;
        }
    }
}

impl Sensor for Lsm6dsv16xI2c {
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
        &[
            // FIXME
        ]
    }

    fn reading_channels(&self) -> ReadingChannels {
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("6-axis IMU")
    }

    fn part_number(&self) -> Option<&'static str> {
        Some(PART_NUMBER)
    }

    fn version(&self) -> u8 {
        0
    }
}
