use serde::{Deserialize, Serialize};

use crate::Conditioned;

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

impl Conditioned for I2cBus {
    fn on(&self) -> Option<&str> {
        self.on.as_deref()
    }

    fn when(&self) -> Option<&str> {
        self.when.as_deref()
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
}

impl Conditioned for I2cPin {
    fn on(&self) -> Option<&str> {
        self.on.as_deref()
    }

    fn when(&self) -> Option<&str> {
        self.when.as_deref()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum I2cFrequency {
    K100,
    K250,
    K400,
}
