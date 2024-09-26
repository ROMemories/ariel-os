//! Provides a sensor abstraction layer.
//!
//! # Definitions
//!
//! In the context of this abstraction:
//!
//! - A *sensor device* is a device measuring one or multiple physical quantities and reporting them as one or more digital values.
//! - Sensor devices measuring the same physical quantity are said to be part of the same *sensor category*.
//!   A sensor device may be part of multiple sensor categories.
//! - A *measurement* is the physical operation of measuring one or several physical quantities.
//! - A *reading* is the result returned by a sensor device after carrying out a measurement.
//!   Values of different physical quantifies can therefore be part of the same reading.
//! - A *sensor driver* refers to a sensor device as exposed by the sensor abstraction layer.
//! - A *sensor driver instance* is an instance of a sensor driver.
//!
//! # For consumers
//!
//! Registered sensor driver instances can be accessed using [`REGISTRY::sensors()`](crate::registry::Registry::sensors).
//!
//! # For implementors
//!
//! Sensor drivers must implement the [`Sensor`] trait.
//!
#![no_std]
// Required by linkme
#![feature(used_with_arg)]
#![feature(trait_upcasting)]
// For watcher tasks
#![feature(type_alias_impl_trait)]
#![deny(unused_must_use)]
#![deny(clippy::pedantic)]

mod category;
pub mod interrupts;
mod label;
mod physical_unit;
mod physical_value;
pub mod registry;
pub mod sensor;
pub mod watcher;

pub use category::Category;
pub use label::Label;
pub use physical_unit::PhysicalUnit;
pub use physical_value::Reading;
pub use registry::{REGISTRY, SENSOR_REFS};
pub use sensor::Sensor;
