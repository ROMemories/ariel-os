#![deny(clippy::pedantic)]

mod utils;

use proc_macro::TokenStream;

include!("main_macro.rs");
include!("thread.rs");
