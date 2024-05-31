#![deny(clippy::pedantic)]
use std::collections::HashMap;

use serde::Deserialize;

#[must_use]
pub fn interrupt_for_peripheral(peripheral: &str) -> Vec<ContextInterrupt> {
    let json = include_str!("./interrupts.json");
    let interrupts: Context = serde_json::from_str(json).unwrap();
    interrupts
        .0
        .iter()
        .filter_map(|(context, interrupts)| {
            if let Some((_, interrupt)) = interrupts.0.iter().find(|(p, _)| *p == peripheral) {
                Some(ContextInterrupt {
                    context: context.to_owned(),
                    interrupt: interrupt.to_owned(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

#[derive(Debug, Deserialize)]
struct Context(HashMap<String, Interrupts>);

#[derive(Debug, Deserialize)]
struct Interrupts(HashMap<String, String>);

#[derive(Debug, Deserialize)]
pub struct ContextInterrupt {
    context: String,
    interrupt: String,
}

impl ContextInterrupt {
    #[must_use]
    pub fn context(&self) -> &str {
        &self.context
    }

    #[must_use]
    pub fn interrupt(&self) -> &str {
        &self.interrupt
    }
}
