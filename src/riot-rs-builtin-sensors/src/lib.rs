//! Collection of built-in sensors implementing the [sensor abstraction layer](riot_rs_sensors).

#![no_std]
#![feature(type_alias_impl_trait)]
#![deny(clippy::pedantic)]

// pub mod aht20;
pub mod lis3dh;
// mod lis3dh_spi;
pub mod push_buttons;
