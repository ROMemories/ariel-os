//! Provides logging facilities.
//!
//! # Syntax of formatting strings
//!
//! The behavior of the provided logging macros depends on which Cargo feature is enabled:
//! - When the `defmt` feature is enabled, `defmt` is used for logging.
//! - When the `log` feature is enabled, `log` is used for logging.
//! - Otherwise, the logging macros are no-ops.
//!
//! This means that the syntax of the formatting strings differs depending on the enabled Cargo
//! feature; please refer to the documentation of those crates for details on the supported syntax.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(nightly, feature(doc_cfg))]
#![cfg_attr(
    all(feature = "log", not(target_has_atomic = "ptr")),
    expect(unsafe_code)
)]
#![deny(missing_docs)]

#[featurecomb::comb]
mod _featurecomb {}

#[allow(unused, reason = "conditional compilation")]
#[doc(hidden)]
#[cfg(feature = "log")]
mod log_logger;

// Make sure the `defmt` logger gets linked.
#[cfg(feature = "esp-println")]
use esp_println as _;

pub use ariel_os_log_facade::*;

/// Prints the panic on the logging output in a consistent manner across loggers.
#[doc(hidden)]
pub fn print_panic(info: &core::panic::PanicInfo<'_>) {
    // `location()`'s documentation currently states that it always returns `Some(_)`.
    // It is unclear what the panic formatting would be otherwise, because the std does not
    // currently handle the case where the location cannot be obtained.
    #[allow(
        unused_variables,
        reason = "FP due to macro usage and conditional compilation"
    )]
    let (location, message) = (info.location().unwrap(), info.message());

    // `PanicMessage` does not currently implement `defmt::Format`.
    // We *need* to use the `Display` implementation and cannot use `PanicMessage::as_str()` as
    // that would not work for dynamically formatted messages.
    #[cfg(feature = "defmt")]
    let message = Display2Format(&message);

    // Mimics the `Display` implementation of `core::panic::PanicInfo`.
    crate::println!("panicked at {}:\n{}", location, message);
}

#[cfg(feature = "log")]
#[doc(hidden)]
pub mod log {
    #[cfg(all(
        context = "ariel-os",
        not(any(feature = "esp-println", feature = "uart"))
    ))]
    pub use ariel_os_debug::debug_output_println as println;

    #[cfg(feature = "esp-println")]
    pub use esp_println::println;

    #[cfg(feature = "uart")]
    pub use crate::uart_println as println;

    /// Prints to the logging output, with a newline.
    #[cfg(not(context = "ariel-os"))]
    #[macro_export]
    macro_rules! noop_println {
        ($($arg:tt)*) => {};
    }
    #[cfg(not(context = "ariel-os"))]
    pub use crate::noop_println as println;
}

#[cfg(feature = "uart")]
#[doc(hidden)]
pub mod backend {
    use embassy_sync::once_lock::OnceLock;

    #[doc(hidden)]
    pub enum Error {
        Writing,
    }

    // Populated by a downstream crate.
    // The function must print to a UART output.
    #[expect(clippy::type_complexity, reason = "not worth it")]
    #[doc(hidden)]
    pub static DEBUG_UART_WRITE_FN: OnceLock<fn(&[u8]) -> Result<(), Error>> = OnceLock::new();

    struct DebugUart;

    impl core::fmt::Write for DebugUart {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();

            if let Some(debug_uart_write_fn) = DEBUG_UART_WRITE_FN.try_get() {
                // Panicking in this case would not be useful as (a) it is recoverable, we would
                // just be dropping some debug output and (b) there would not be a output to print
                // the panic on, as there can currently only be one backend at once.
                let _ = debug_uart_write_fn(bytes);
            }

            Ok(())
        }
    }

    // Based on <https://blog.m-ou.se/format-args/>.
    #[doc(hidden)]
    pub fn _print(args: core::fmt::Arguments<'_>) {
        use core::fmt::Write as _;

        DebugUart.write_fmt(args).unwrap();
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! uart_println {
        ($($arg:tt)*) => {{
            #[expect(clippy::used_underscore_items, reason = "consistency with std::println")]
            $crate::backend::_print(format_args!("{}\n", format_args!($($arg)*)));
        }};
    }
}

#[doc(hidden)]
pub fn init() {
    #[cfg(feature = "log")]
    log_logger::init();
}
