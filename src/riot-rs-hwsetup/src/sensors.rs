use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{derive_conditioned, peripherals::Peripherals, Conditioned};

pub use serde_yaml::Number as YamlNumber;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sensor {
    name: String,
    driver: String,
    label: String,
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
    pub fn label(&self) -> &str {
        &self.label
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

derive_conditioned!(Sensor);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorConfig(HashMap<String, SensorConfigValue>);

impl SensorConfig {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SensorConfigValue)> {
        self.0.iter()
    }
}

// TODO: add a TypePath variant and provide a custom Deserialize implementation
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SensorConfigValue {
    String(String),
    Number(YamlNumber), // TODO: replace this with our own Number type
    Bool(bool),
}

#[derive(Debug)]
pub enum StringOrTypePath<'a> {
    String(&'a str),
    TypePath(&'a str),
}

impl<'a> StringOrTypePath<'a> {
    #[must_use]
    pub fn from(value: &'a str) -> Self {
        let is_type_path_config_string =
            value.len() >= 2 && value.starts_with('@') && !value.starts_with("@@");

        if is_type_path_config_string {
            #[expect(
                clippy::indexing_slicing,
                reason = "a type path string always has at least two bytes"
            )]
            StringOrTypePath::TypePath(&value[1..])
        } else if let Some(stripped) = value.strip_prefix('@') {
            // Discard the first @
            StringOrTypePath::String(stripped)
        } else {
            StringOrTypePath::String(value)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SensorBus {
    I2c(HashMap<String, SensorBusI2c>),
    Spi(HashMap<String, SensorBusSpi>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorBusI2c {
    on: Option<String>,
    when: Option<String>,
    // TODO: we may want to add additional per-sensor configuration later (e.g., I2C frequency
    // configuration)
}

derive_conditioned!(SensorBusI2c);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorBusSpi {
    cs: Vec<crate::buses::spi::Pin>,
    on: Option<String>,
    when: Option<String>,
    // TODO: we may want to add additional per-sensor configuration later
}

impl SensorBusSpi {
    #[must_use]
    pub fn cs(&self) -> &[crate::buses::spi::Pin] {
        &self.cs
    }
}

derive_conditioned!(SensorBusSpi);
