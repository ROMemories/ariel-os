use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buses {
    i2c: Vec<i2c::Bus>,
    spi: Vec<spi::Bus>,
}

impl Buses {
    #[must_use]
    pub fn i2c(&self) -> &[i2c::Bus] {
        &self.i2c
    }

    #[must_use]
    pub fn spi(&self) -> &[spi::Bus] {
        &self.spi
    }
}

pub mod i2c {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::{derive_conditioned, Conditioned};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Bus {
        name: String,
        peripheral: HashMap<String, BusPeripheral>, // FIXME: require at least one element
    }

    impl Bus {
        #[must_use]
        pub fn name(&self) -> &str {
            &self.name
        }

        #[must_use]
        pub fn peripheral(&self) -> &HashMap<String, BusPeripheral> {
            &self.peripheral
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BusPeripheral {
        on: Option<String>,
        when: Option<String>,
        sda: Vec<Pin>, // FIXME: require at least one element
        scl: Vec<Pin>, // FIXME: require at least one element
        frequency: Frequency,
    }

    impl BusPeripheral {
        #[must_use]
        pub fn sda(&self) -> &[Pin] {
            &self.sda
        }

        #[must_use]
        pub fn scl(&self) -> &[Pin] {
            &self.scl
        }

        #[must_use]
        pub fn frequency(&self) -> Frequency {
            self.frequency
        }
    }

    derive_conditioned!(BusPeripheral);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Pin {
        pin: String,
        pull_up: bool,
        on: Option<String>,
        when: Option<String>,
    }

    impl Pin {
        #[must_use]
        pub fn pin(&self) -> &str {
            &self.pin
        }

        #[must_use]
        pub fn pull_up(&self) -> bool {
            self.pull_up
        }
    }

    derive_conditioned!(Pin);

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Frequency {
        K100,
        K250,
        K400,
    }
}

pub mod spi {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::{derive_conditioned, Conditioned};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Bus {
        name: String,
        peripheral: HashMap<String, BusPeripheral>, // FIXME: require at least one element
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BusPeripheral {
        on: Option<String>,
        when: Option<String>,
        sck: Vec<Pin>, // FIXME: require at least one element
        miso: Vec<Pin>, // FIXME: require at least one element
        mosi: Vec<Pin>, // FIXME: require at least one element
        mode: Mode,
    }

    impl BusPeripheral {
        #[must_use]
        pub fn sck(&self) -> &[Pin] {
            &self.sck
        }

        #[must_use]
        pub fn miso(&self) -> &[Pin] {
            &self.miso
        }

        #[must_use]
        pub fn mosi(&self) -> &[Pin] {
            &self.mosi
        }
    }

    derive_conditioned!(BusPeripheral);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Pin {
        pin: String,
        on: Option<String>,
        when: Option<String>,
    }

    impl Pin {
        #[must_use]
        pub fn pin(&self) -> &str {
            &self.pin
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Mode {
        Mode0,
        Mode1,
        Mode2,
        Mode3,
    }
}
