use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_hal::digital::InputPin;
use riot_rs_embassy::Spawner;
use riot_rs_embassy_common::maybe_inverted_pin::MaybeInvertedPin;
use riot_rs_sensors::{
    sensor::{
        Accuracy, Mode, ReadingAxes, ReadingAxis, ReadingError, ReadingWaiter, SetModeError, State,
        TriggerMeasurementError, Value, Values,
    },
    sensor_signaling::SensorSignaling,
    state_atomic::StateAtomic,
    Category, Label, MeasurementUnit, Sensor,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    pub active_low: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { active_low: true }
    }
}

pub type PushButton = GenericPushButton<riot_rs_embassy::gpio::Input>;

// TODO: how to name this?
// TODO: is it useful to expose this or should we just make it non-generic?
pub struct GenericPushButton<I: InputPin> {
    state: StateAtomic,
    label: Option<&'static str>,
    button: Mutex<CriticalSectionRawMutex, Option<MaybeInvertedPin<I>>>, // TODO: maybe use MaybeUninit
    signaling: SensorSignaling,
}

impl<I: InputPin + 'static> GenericPushButton<I> {
    #[allow(clippy::new_without_default)]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            button: Mutex::new(None),
            signaling: SensorSignaling::new(),
        }
    }

    // TODO: add Spawner for consistency
    pub fn init(&'static self, _spawner: Spawner, gpio: I, config: Config) {
        if self.state.get() == State::Uninitialized {
            {
                // We use `try_lock()` instead of `lock()` to not make this function async.
                // This mutex cannot be locked at this point as it is private and can only be
                // locked when the sensor has been initialized successfully.
                let mut button = self.button.try_lock().unwrap();
                *button = Some(MaybeInvertedPin::new(gpio, !config.active_low));
            }

            self.state.set(State::Enabled);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            let is_pressed = self.button.lock().await.as_mut().unwrap().is_low().unwrap();

            self.signaling
                .signal_reading(Values::V1([Value::new(
                    i32::from(is_pressed),
                    Accuracy::NoError,
                )]))
                .await;
        }
    }
}

impl<I: InputPin + Send + 'static> Sensor for GenericPushButton<I> {
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

    fn set_mode(&self, mode: Mode) -> Result<State, SetModeError> {
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
        &[Category::PushButton]
    }

    fn reading_axes(&self) -> ReadingAxes {
        ReadingAxes::V1([ReadingAxis::new(Label::Main, 0, MeasurementUnit::Bool)])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("push button")
    }

    fn part_number(&self) -> Option<&'static str> {
        None
    }

    fn version(&self) -> u8 {
        0
    }
}
