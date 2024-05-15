use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{derive_conditioned, Conditioned};

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
    name: String,
    peripheral: HashMap<String, I2cBusPeripheral>,
}

impl I2cBus {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn peripheral(&self) -> &HashMap<String, I2cBusPeripheral> {
        &self.peripheral
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct I2cBusPeripheral {
    on: Option<String>,
    when: Option<String>,
    sda: Vec<I2cPin>,
    scl: Vec<I2cPin>,
    frequency: I2cFrequency,
}

impl I2cBusPeripheral {
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

derive_conditioned!(I2cBusPeripheral);

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

derive_conditioned!(I2cPin);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum I2cFrequency {
    K100,
    K250,
    K400,
}
