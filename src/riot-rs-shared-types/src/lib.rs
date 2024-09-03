//! Architecture-agnostic types shared between architectures.
#![feature(lint_reasons)]
#![no_std]
#![feature(doc_auto_cfg)]
#![deny(missing_docs)]

pub mod gpio;

#[cfg(context = "cortex-m")]
pub mod executor_swi;

#[cfg(feature = "i2c")]
pub mod i2c;

pub use embassy_futures;
pub use embassy_time;
