//! riot-rs
//!
//! This is a meta-package, pulling in the sub-crates of RIOT-rs.

#![no_std]

#[doc(inline)]
pub use riot_rs_buildinfo as buildinfo;
#[doc(inline)]
pub use riot_rs_debug as debug;
#[doc(inline)]
pub use riot_rs_embassy as embassy;
pub use riot_rs_embassy::define_peripherals;
#[doc(inline)]
pub use riot_rs_rt as rt;

// Attribute macros
pub use riot_rs_macros::config;
#[cfg(feature = "threading")]
pub use riot_rs_macros::thread;

#[cfg(feature = "threading")]
pub use riot_rs_threads as thread;

// These are used by proc-macros we provide
pub use linkme;
pub use static_cell;

// ensure this gets linked
use riot_rs_boards as _;
