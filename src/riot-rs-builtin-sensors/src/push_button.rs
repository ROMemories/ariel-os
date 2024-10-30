use embassy_sync::once_lock::OnceLock;
use riot_rs_embassy::{
    gpio::{Input, MaybeInvertedInput},
    Spawner,
};
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
    /// Whether the push button pulls the input pin low when pressed.
    pub active_low: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { active_low: true }
    }
}

pub struct PushButton {
    state: StateAtomic,
    label: Option<&'static str>,
    button: OnceLock<MaybeInvertedInput>,
    signaling: SensorSignaling,
}

impl PushButton {
    #[allow(clippy::new_without_default)]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            button: OnceLock::new(),
            signaling: SensorSignaling::new(),
        }
    }

    pub fn init(&self, _spawner: Spawner, pin: Input, config: Config) {
        if self.state.get() == State::Uninitialized {
            let _ = self
                .button
                .init(MaybeInvertedInput::new(pin, !config.active_low));

            self.state.set(State::Enabled);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            self.signaling.wait_for_trigger().await;

            let is_pressed = self.button.get().await.is_low();

            self.signaling
                .signal_reading(Values::V1([Value::new(
                    i32::from(is_pressed),
                    Accuracy::NoError,
                )]))
                .await;
        }
    }
}

impl Sensor for PushButton {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_type_sizes() {
        assert_eq!(size_of::<Values>(), 6 * size_of::<u32>());
        assert_eq!(size_of::<PushButton>(), 30 * size_of::<u32>());
    }
}
