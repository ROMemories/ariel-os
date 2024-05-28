//! Reads and parses the hardware setup defined in a configuration file.

#![feature(lint_reasons)]
#![deny(clippy::pedantic)] // FIXME: remove this when rebasing

pub mod buses;
pub mod peripherals;
pub mod sensors;

use std::{
    env, fs,
    io::{self, Read},
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{buses::Buses, sensors::Sensor};

const PATH_ENV_VAR: &str = "SETUP_FILE";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HwSetup {
    buses: Buses,
    sensors: Sensors,
}

impl HwSetup {
    /// Returns the file path of the setup file read from the `SETUP_FILE` environment variable.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EnvVarNotFound`] if the environment variable from which to read the file
    /// path of the setup file was not present in the environment.
    pub fn get_path_from_env() -> Result<PathBuf, Error> {
        Ok(PathBuf::from(env::var(PATH_ENV_VAR).map_err(|_| {
            Error::EnvVarNotFound {
                var: PATH_ENV_VAR.to_owned(),
            }
        })?))
    }

    /// Parses a [`HwSetup`] struct from a file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::SetupFileNotFound`] if the file cannot be found.
    pub fn read_from_path(path: &Path) -> Result<Self, Error> {
        let file = fs::File::open(path).map_err(|source| Error::SetupFileNotFound {
            path: path.to_path_buf(),
            source,
        })?;
        Self::read_from_reader(&file)
    }

    /// Parses a [`HwSetup`] struct from a reader.
    ///
    /// # Errors
    ///
    /// Returns [`Error::YamlParsing`] in case of parsing error.
    pub fn read_from_reader(setup: impl Read) -> Result<Self, Error> {
        let hwconfig =
            serde_yaml::from_reader(setup).map_err(|source| Error::YamlParsing { source })?;

        Ok(hwconfig)
    }

    #[must_use]
    pub fn buses(&self) -> &Buses {
        &self.buses
    }

    #[must_use]
    pub fn sensors(&self) -> &Sensors {
        &self.sensors
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sensors {
    #[serde(default)]
    max_reading_value_count: Option<MaxReadingValueCount>,
    connected: Vec<Sensor>,
}

impl Sensors {
    #[must_use]
    pub fn max_reading_value_count(&self) -> NonZeroU8 {
        self.max_reading_value_count.unwrap_or_default().0
    }

    #[must_use]
    pub fn connected(&self) -> &[Sensor] {
        &self.connected
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MaxReadingValueCount(NonZeroU8);

impl Default for MaxReadingValueCount {
    fn default() -> Self {
        Self(NonZeroU8::new(3).unwrap())
    }
}

// TODO
#[derive(Debug)]
pub enum Error {
    EnvVarNotFound { var: String },
    SetupFileNotFound { path: PathBuf, source: io::Error },
    YamlParsing { source: serde_yaml::Error },
}

/// This trait is sealed and cannot be implemented for types outside this crate.
#[allow(private_bounds, reason = "sealed trait")]
pub trait Conditioned: private::Sealed {
    #[must_use]
    fn on(&self) -> Option<&str>;

    #[must_use]
    fn when(&self) -> Option<&str>;
}

mod private {
    pub(crate) trait Sealed {}
}

macro_rules! derive_conditioned {
    ($type:ident) => {
        impl crate::private::Sealed for $type {}

        impl Conditioned for $type {
            fn on(&self) -> Option<&str> {
                self.on.as_deref()
            }

            fn when(&self) -> Option<&str> {
                self.when.as_deref()
            }
        }
    };
}
pub(crate) use derive_conditioned;
