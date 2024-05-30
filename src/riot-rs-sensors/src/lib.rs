//! Provides a sensor abstraction layer.
//!
//! Sensors must implement the [`Sensor`] trait, and must be registered into the
//! [`static@SENSOR_REFS`] [distributed slice](linkme).

#![no_std]
// Required by linkme
#![feature(used_with_arg)]
#![feature(error_in_core)]
#![feature(trait_upcasting)]
#![deny(unused_must_use)]
#![deny(clippy::pedantic)]

mod category;
mod label;
mod physical_unit;
pub mod registry;
pub mod sensor;

pub use category::Category;
pub use label::Label;
pub use physical_unit::PhysicalUnit;
pub use registry::REGISTRY;
pub use sensor::{Reading, Sensor};

// FIXME: this should not be part of the users' documentation, to force users to use
// `Registry::sensors()` instead
pub use registry::SENSOR_REFS;
