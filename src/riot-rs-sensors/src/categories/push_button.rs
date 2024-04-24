use core::future::Future;

use crate::{
    sensor::{PhysicalValue, ReadingResult},
    Sensor,
};

#[allow(clippy::module_name_repetitions)]
pub trait PushButtonSensor: Sensor {
    fn read_press_state(&self) -> impl Future<Output = ReadingResult<PushButtonReading>>;
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PushButtonReading {
    value: PhysicalValue,
}

impl PushButtonReading {
    #[must_use]
    pub fn new(value: PhysicalValue) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn is_pressed(&self) -> bool {
        self.value.value() != 0
    }
}
