//! Collection of built-in sensors implementing the [sensor abstraction layer](riot_rs_sensors).

#![no_std]
#![feature(lint_reasons)]
#![deny(clippy::pedantic)]

pub mod lis3dh;
pub mod push_buttons;
