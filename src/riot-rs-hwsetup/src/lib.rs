//! Reads and parses the hardware setup defined in a configuration file.

#![deny(clippy::pedantic)]

use std::{collections::HashMap, env, fs, path::PathBuf};

use serde::{Serialize, Deserialize};
use serde_yaml::Value as YamlValue;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HwSetup {
    buses: Buses,
    sensors: Vec<Sensor>,
}

impl HwSetup {
    pub fn read_from_file() -> Result<Self, Error> {
        // let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()); // FIXME: do something about this error?
        // FIXME
        let root = PathBuf::from("examples/embassy-http-server"); // FIXME: do something about this error?
        let file_path = root.join("hw-setup.yml");

        let file = fs::File::open(file_path).unwrap(); // FIXME: handle the error
        let hwconfig = serde_yaml::from_reader(&file).unwrap(); // FIXME: handle the error

        Ok(hwconfig)
    }

    #[must_use]
    pub fn buses(&self) -> &Buses {
        &self.buses
    }

    #[must_use]
    pub fn sensors(&self) -> &[Sensor] {
        &self.sensors
    }
}

// TODO
#[derive(Debug)]
pub enum Error {
    ConfigNotFound,
    YamlError,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buses {
    i2c: Vec<I2cBus>,
}

impl Buses {
    #[must_use]
    pub fn i2c(&self) -> &[I2cBus] {
        &self.i2c
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct I2cBus {
    instance: String,
    on: Option<String>,
    when: Option<String>,
    sda: Vec<I2cPin>,
    scl: Vec<I2cPin>,
    frequency: I2cFrequency,
}

impl I2cBus {
    #[must_use]
    pub fn on(&self) -> Option<&str> {
        self.on.as_deref()
    }

    #[must_use]
    pub fn when(&self) -> Option<&str> {
        self.when.as_deref()
    }

    #[must_use]
    pub fn sda(&self) -> &[I2cPin] {
        &self.sda
    }

    #[must_use]
    pub fn scl(&self) -> &[I2cPin] {
        &self.scl
    }

    #[must_use]
    pub fn frequency(&self) -> I2cFrequency {
        self.frequency
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct I2cPin {
    pin: String,
    pull_up: bool,
    on: Option<String>,
    when: Option<String>,
}

impl I2cPin {
    #[must_use]
    pub fn pin(&self) -> &str {
        &self.pin
    }

    #[must_use]
    pub fn pull_up(&self) -> bool {
        self.pull_up
    }

    #[must_use]
    pub fn on(&self) -> Option<&str> {
        self.on.as_deref()
    }

    #[must_use]
    pub fn when(&self) -> Option<&str> {
        self.when.as_deref()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum I2cFrequency {
    K100,
    K250,
    K400,
}

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
    pub fn on(&self) -> Option<&str> {
        self.on.as_deref()
    }

    #[must_use]
    pub fn when(&self) -> Option<&str> {
        self.when.as_deref()
    }

    #[must_use]
    pub fn peripherals(&self) -> Option<&Peripherals> {
        self.peripherals.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorConfig(HashMap<String, YamlValue>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SensorBus {
    #[serde(rename = "i2c")]
    I2c(Vec<SensorBusI2c>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorBusI2c {
    on: Option<String>,
    instance: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peripherals(HashMap<String, Vec<Peripheral>>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peripheral {
    instance: String,
    on: Option<String>,
}
