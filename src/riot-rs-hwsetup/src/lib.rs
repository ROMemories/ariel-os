//! Reads and parses the hardware setup defined in a configuration file.

#![deny(clippy::pedantic)]

pub mod buses;
pub mod peripherals;
pub mod sensors;

use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{buses::Buses, sensors::Sensor};

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

pub trait Conditioned {
    #[must_use]
    fn on(&self) -> Option<&str>;

    #[must_use]
    fn when(&self) -> Option<&str>;
}
