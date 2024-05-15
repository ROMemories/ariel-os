use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;

use crate::{peripherals::Peripherals, Conditioned};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sensor {
    name: String,
    driver: String,
    on: Option<String>,
    when: Option<String>,
    with: Option<SensorConfig>,
    bus: Option<SensorBus>,
    peripherals: Option<Peripherals>,
}

impl Sensor {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn driver(&self) -> &str {
        &self.driver
    }

    #[must_use]
    pub fn with(&self) -> Option<&SensorConfig> {
        self.with.as_ref()
    }

    #[must_use]
    pub fn bus(&self) -> Option<&SensorBus> {
        self.bus.as_ref()
    }

    #[must_use]
    pub fn peripherals(&self) -> Option<&Peripherals> {
        self.peripherals.as_ref()
    }
}

impl Conditioned for Sensor {
    fn on(&self) -> Option<&str> {
        self.on.as_deref()
    }

    fn when(&self) -> Option<&str> {
        self.when.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorConfig(HashMap<String, YamlValue>);

impl SensorConfig {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &YamlValue)> {
        self.0.iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SensorBus {
    I2c(Vec<SensorBusI2c>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorBusI2c {
    on: Option<String>,
    instance: String,
}
