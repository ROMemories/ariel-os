#![deny(clippy::pedantic)]

mod utils;

use proc_macro::TokenStream;

include!("config.rs");
include!("define_count_adjusted_enums.rs");
include!("hw_setup.rs");
include!("hw_setup_init.rs");
include!("measure_sensor.rs");
include!("spawner.rs");
include!("task.rs");
include!("thread.rs");
