//! Collection of built-in sensors implementing the [sensor abstraction layer](riot_rs_sensors).

#![no_std]
#![feature(type_alias_impl_trait)]
// #![deny(missing_docs)]
#![deny(clippy::pedantic)]

pub mod lsm303agr;
pub mod push_button;
