use core::future::Future;

use crate::{
    sensor::{PhysicalValue, ReadingResult},
    Sensor,
};

#[allow(clippy::module_name_repetitions)]
pub trait TemperatureSensor: Sensor {
    fn read_temperature(&self) -> impl Future<Output = ReadingResult<TemperatureReading>>;
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct TemperatureReading {
    value: PhysicalValue,
}

impl TemperatureReading {
    #[must_use]
    pub fn new(value: PhysicalValue) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn temperature(&self) -> PhysicalValue {
        self.value
    }
}
