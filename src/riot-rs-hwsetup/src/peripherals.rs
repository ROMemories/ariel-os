use serde::{Deserialize, Serialize};

use crate::{derive_conditioned, Conditioned};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peripherals(Vec<Peripheral>);

impl Peripherals {
    #[must_use]
    pub fn get(&self) -> &[Peripheral] {
        &self.0
    }

    pub fn inputs(&self) -> impl Iterator<Item = &Input> {
        self.0.iter().filter_map(|p| {
            if let Peripheral::Input(input) = p {
                Some(input)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Peripheral {
    Input(Input),
    Output(Output),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pin: String,
    on: Option<String>,
    when: Option<String>,
    pull: PullResistor,
}

impl Input {
    #[must_use]
    pub fn pin(&self) -> &str {
        &self.pin
    }

    #[must_use]
    pub fn pull(&self) -> PullResistor {
        self.pull
    }
}

derive_conditioned!(Input);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pin: String,
    on: Option<String>,
    when: Option<String>,
}

derive_conditioned!(Output);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullResistor {
    Up,
    Down,
    None,
}
