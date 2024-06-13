use portable_atomic::{AtomicU8, Ordering};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embedded_hal::digital::InputPin;
use riot_rs_embassy::Spawner;

use riot_rs_sensors::{
    sensor::{
        AccuracyError, Mode, PhysicalValue, PhysicalValues, ReadingError, ReadingInfo,
        ReadingInfos, ReadingResult, State,
    },
    Category, Label, PhysicalUnit, Sensor,
};

// TODO: allow to set whether this is active low or active high
#[derive(Debug)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

pub type PushButton = GenericPushButton<riot_rs_embassy::arch::gpio::Input<'static>>;

// TODO: how to name this?
// TODO: is it useful to expose this or should we just make it non-generic?
pub struct GenericPushButton<I: InputPin> {
    state: AtomicU8,
    label: Option<&'static str>,
    // buttons: [Option<Button>; N], // TODO: maybe use MaybeUninit
    button: Mutex<CriticalSectionRawMutex, Option<I>>, // TODO: maybe use MaybeUninit
}

impl<I: InputPin + 'static> GenericPushButton<I> {
    #[allow(clippy::new_without_default)]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            state: AtomicU8::new(State::Uninitialized as u8),
            label,
            button: Mutex::new(None),
        }
    }

    // TODO: add Spawner for consistency
    pub fn init(&'static self, _spawner: Spawner, gpio: I, config: Config) {
        if self.state.load(Ordering::Acquire) == State::Uninitialized as u8 {
            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut button = self.button.try_lock().unwrap();
            *button = Some(gpio);

            self.state.store(State::Enabled as u8, Ordering::Release);
        }
    }
}

impl<I: InputPin + Send + 'static> Sensor for GenericPushButton<I> {
    #[allow(refining_impl_trait)]
    async fn measure(&self) -> ReadingResult<PhysicalValues> {
        if self.state.load(Ordering::Acquire) != State::Enabled as u8 {
            return Err(ReadingError::NonEnabled);
        }

        let reading = self.button.lock().await.as_mut().unwrap().is_low().unwrap();

        // FIXME: this has to be configurable to handle both active-low and active-high push button
        // inputs
        let is_pressed = reading;

        Ok(PhysicalValues::V1([PhysicalValue::new(
            i32::from(is_pressed),
            AccuracyError::None,
        )]))
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
        &[Category::PushButton]
    }

    fn reading_infos(&self) -> ReadingInfos {
        ReadingInfos::V1([ReadingInfo::new(Label::Main, 0, PhysicalUnit::ActiveOne)])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("Push button")
    }

    fn part_number(&self) -> &'static str {
        "push button"
    }

    fn version(&self) -> u8 {
        0
    }
}
