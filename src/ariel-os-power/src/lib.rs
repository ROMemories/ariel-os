//! Provides power management functionality.

#![deny(missing_docs)]
#![cfg_attr(not(context = "native"), no_std)]

mod reset;

pub mod stop_mode;

use ariel_os_hal::hal::power::WakeupInterrupts;

pub use reset::*;

/// Enters standby mode.
///
/// In this mode, almost every clock of the microcontroller is off, and the RAM is powered off when
/// possible, requiring rebooting the application completely when waking up.
/// This function never returns to represent that.
///
/// # Important note
///
/// This is currently implemented on a best-effort basis.
/// Some microcontrollers may not support these low-power settings, they may not be implemented
/// yet, or they may be lacking testing.
/// Do measure the power consumption of your hardware when relevant for your application.
///
/// # Wake-up conditions
///
/// The conditions allowing to trigger a wake-up depend on the hardware.
/// The `WakeupInterrupts` type is used to configure which interrupts can trigger a wake-up.
/// This type is HAL-specific and can be found in `ariel_os::hal::power`.
/// On some hardware, it may however not be possible to prevent specific interrupts from triggering
/// a wake-up.
///
/// If the hardware does not support triggering a reset on wake-up, or if all the wake-up
/// conditions are disabled through `WakeupInterrupts`, this function functionally powers down the
/// microcontroller, with no ability to automatically wake up (except from a hardware reset).
pub fn enter_standby_mode(interrupts: WakeupInterrupts) -> ! {
    ariel_os_hal::hal::power::enter_standby_mode(interrupts);

    // This loop will not be executed, this is only to satisfy the return type.
    #[allow(clippy::empty_loop, reason = "for platform-independent tooling only")]
    loop {
        cfg_select! {
            context = "cortex-m" => cortex_m::asm::wfi(),
            _ => {}
        }
    }
}
